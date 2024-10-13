#![allow(unused_imports)]
use html2md_rs::{
    parser::safe_parse_html,
    structs::{AttributeValues, Node, NodeType},
};
use std::{collections::HashMap, io::Write};

const LOG_FILE_PATH: &str = "log.xml";

/// total event count: 78320

#[allow(dead_code, unused_variables, non_snake_case)]
#[derive(Default, Debug, Clone)]
struct EventLogon {
    EventID: usize,
    /// ISO 8601 format
    TimeCreated: String,
    SubjectUserSid: String,
    SubjectUserName: String,
    SubjectDomainName: String,
    SubjectLogonId: String,
    TargetUserSid: String,
    TargetUserName: String,
    TargetDomainName: String,
    TargetLogonId: String,
}

fn main() -> std::io::Result<()> {
    let input = std::fs::read_to_string(LOG_FILE_PATH)?;

    let Ok(tree) = safe_parse_html(input) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to parse HTML",
        ));
    };

    // println!("{:#?}", tree);

    let events = tree.children;

    let mut events_4624 = Vec::new();
    let mut events_4625 = Vec::new();

    for event in events.iter() {
        let system = event
            .children
            .iter()
            .find(|n| n.tag_name == Some(NodeType::Unknown("system".to_string())))
            .unwrap();
        let Some(event_data) = event
            .children
            .iter()
            .find(|n| n.tag_name == Some(NodeType::Unknown("eventdata".to_string())))
        else {
            continue;
        };

        let event_id = system
            .children
            .iter()
            .find(|n| n.tag_name == Some(NodeType::Unknown("eventid".to_string())))
            .unwrap()
            .children
            .first()
            .unwrap()
            .value
            .clone()
            .unwrap();

        let event = EventLogon {
            EventID: event_id.parse().unwrap_or_default(),
            TimeCreated: system
                .children
                .iter()
                .find(|n| n.tag_name == Some(NodeType::Unknown("timecreated".to_string())))
                .and_then(|n| n.attributes.as_ref())
                .and_then(|attrs| attrs.get("SystemTime"))
                .map(|s| s.to_string())
                .unwrap_or_default(),
            SubjectUserSid: get_attribute(event_data, "SubjectUserSid"),
            SubjectUserName: get_attribute(event_data, "SubjectUserName"),
            SubjectDomainName: get_attribute(event_data, "SubjectDomainName"),
            SubjectLogonId: get_attribute(event_data, "SubjectLogonId"),
            TargetUserName: get_attribute(event_data, "TargetUserName"),
            TargetDomainName: get_attribute(event_data, "TargetDomainName"),
            TargetUserSid: get_attribute(event_data, "TargetUserSid"),
            TargetLogonId: get_attribute(event_data, "TargetLogonId"),
        };

        if event_id == "4624" {
            events_4624.push(event.clone());
        } else if event_id == "4625" {
            events_4625.push(event.clone());
        }
    }

    let mut event_4624_users_frequncy: HashMap<String, [usize; 24]> = HashMap::new();

    for event in events_4624.iter() {
        let time = event.TimeCreated.split('T').nth(1).unwrap();
        let hour = time.split(':').next().unwrap().parse::<usize>().unwrap();
        event_4624_users_frequncy
            .entry(event.TargetUserName.clone())
            .or_insert([0; 24])[hour] += 1;
    }

    let mut sorted_event_4624_users_frequncy: Vec<_> = event_4624_users_frequncy.iter().collect();
    sorted_event_4624_users_frequncy
        .sort_by(|a, b| b.1.iter().sum::<usize>().cmp(&a.1.iter().sum::<usize>()));

    // write to file as csv
    let mut file = std::fs::File::create("frequency.csv")?;

    // header is Time, user1, user2, user3, ...
    let mut header = "Time,".to_string();
    for (user, _) in sorted_event_4624_users_frequncy.iter() {
        header.push_str(&format!("{},", user));
    }
    header.push('\n');
    file.write_all(header.as_bytes())?;

    // write each user's frequency at each hour
    for i in 0..24 {
        let mut line = format!("{},", i);
        for (_, freq) in sorted_event_4624_users_frequncy.iter() {
            line.push_str(&format!("{},", freq[i]));
        }
        line.push('\n');
        file.write_all(line.as_bytes())?;
    }

    let mut events_4624_users = HashMap::new();

    for event in events_4624.iter() {
        *events_4624_users
            .entry(event.TargetUserName.clone())
            .or_insert(0) += 1;
    }

    let mut sorted_events_4624_users: Vec<_> = events_4624_users.iter().collect();
    sorted_events_4624_users.sort_by(|a, b| b.1.cmp(a.1));

    println!("users: {:?}", sorted_events_4624_users);
    print!("user count: {}", sorted_events_4624_users.len());

    println!("event 4625 count: {}", events_4625.len());
    println!("events 4625: {:#?}", events_4625);

    Ok(())
}

fn get_attribute(event_data: &Node, data_name: &str) -> String {
    event_data
        .children
        .iter()
        .find(|n| {
            n.attributes.as_ref().map_or(false, |attrs| {
                attrs.get("Name") == Some(AttributeValues::String(format!("'{}'", data_name)))
            })
        })
        .and_then(|n| n.children.first())
        .and_then(|n| n.value.clone())
        .unwrap_or_default()
}
