use std::collections::HashMap;
use std::ops::Add;
use std::time::SystemTime;
use rand::Rng;
use crate::config::{DAY, HOUR, MINUTE, WEEK};

#[derive(Debug)]
pub(crate) struct ClipboardTampering {
    pub(crate) tamper: fn(&mut String),
    pub(crate) trigger: SystemTime,
    pub(crate) enabled: bool,
    pub(crate) cooldown: u32,
}

pub(crate) fn get_tampering_functions() -> HashMap<String, ClipboardTampering> {
    let mut tampering_functions = HashMap::new();
    tampering_functions.insert(String::from("semicolon_to_greek_question_mark"), ClipboardTampering {
        tamper: semicolon_to_greek_question_mark,
        trigger: get_initial_tampering_trigger(),
        enabled: true,
        cooldown: 1 * DAY,
    });
    tampering_functions.insert(String::from("swap_case"), ClipboardTampering {
        tamper: swap_case,
        trigger: get_initial_tampering_trigger(),
        enabled: true,
        cooldown: 1 * WEEK,
    });
    tampering_functions.insert(String::from("to_uppercase"), ClipboardTampering {
        tamper: to_uppercase,
        trigger: get_initial_tampering_trigger(),
        enabled: true,
        cooldown: 3 * DAY,
    });
    tampering_functions.insert(String::from("to_lowercase"), ClipboardTampering {
        tamper: to_lowercase,
        trigger: get_initial_tampering_trigger(),
        enabled: true,
        cooldown: 5 * DAY,
    });
    tampering_functions.insert(String::from("reverse_string"), ClipboardTampering {
        tamper: reverse_string,
        trigger: get_initial_tampering_trigger(),
        enabled: true,
        cooldown: 10 * DAY,
    });

    tampering_functions
}

fn get_initial_tampering_trigger() -> SystemTime {
    SystemTime::now().add(std::time::Duration::from_secs(rand::thread_rng().gen_range(5 * MINUTE..4 * HOUR) as u64))
}

// TODO function to remove all newlines (or a random newline) from the clipboard content

fn semicolon_to_greek_question_mark(s: &mut String) {
    let replaced = s.replace(";", ";");
    *s = replaced;
}

fn swap_case(s: &mut String) {
    let swapped: String = s.chars().map(|c| {
        if c.is_uppercase() {
            c.to_lowercase().next().unwrap()
        } else {
            c.to_uppercase().next().unwrap()
        }
    }).collect();

    *s = swapped;
}

fn to_uppercase(s: &mut String) {
    s.make_ascii_uppercase();
}

fn to_lowercase(s: &mut String) {
    s.make_ascii_lowercase();
}

fn reverse_string(s: &mut String) {
    let reversed: String = s.chars().rev().collect();
    *s = reversed;
}