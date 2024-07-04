use anyhow::anyhow;
use chrono::NaiveDateTime;

pub fn parse_from_markdown(content: &str) -> anyhow::Result<Vec<(&str, &str)>> {
    let mut vec = vec![];
    // read content line by line
    for line in content.lines() {
        if line.starts_with("<!--[") && line.ends_with("]-->") {
            if let Some(i) = line.find('|') {
                let key = &line[5..i];
                let value = &line[i + 1..line.len() - 4];
                vec.push((key, value))
            }
        }
    }
    Ok(vec)
}

pub fn find_single(map: &[(&str, &str)], key: &str) -> anyhow::Result<String> {
    Ok(map
        .iter()
        .find(|i| i.0 == key)
        .ok_or_else(|| anyhow!("Couldn't find value with key '{}'", key))?
        .1
        .to_string())
}

pub fn find_multiple(map: &[(&str, &str)], key: &str) -> Vec<String> {
    map.iter()
        .filter(|i| i.0 == key)
        .map(|i| i.1.to_string())
        .collect()
}

pub fn find_timestamp(map: &[(&str, &str)], key: &str) -> anyhow::Result<NaiveDateTime> {
    let value = map
        .iter()
        .find(|i| i.0 == key)
        .ok_or_else(|| anyhow!("Couldn't find value with key '{}'", key))?
        .1;

    let int = i64::from_str_radix(value, 10)?;

    NaiveDateTime::from_timestamp_opt(int, 0)
        .ok_or_else(|| anyhow!("Error while generating timestamp"))
}

pub fn calculate_read_time(content: &str) -> u32 {
    // TODO: improve this maybe idk seems fine for now
    const AVG_CHAR_PER_WORD: usize = 5;
    const AVG_WPM: usize = 238;
    (content.len() / (AVG_CHAR_PER_WORD * AVG_WPM)) as u32
}
