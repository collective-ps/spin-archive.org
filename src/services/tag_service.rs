use std::collections::HashSet;

use diesel::SqliteConnection;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::models::tag::{self, NewTag, Tag};
use crate::models::upload::UploadStatus;
use crate::schema::uploads;

pub use crate::models::tag::{all, by_names};

pub fn create_from_tag_string(conn: &SqliteConnection, tag_string: &str) {
    let tags = sanitize_tags(tag_string);

    for tag_name in tags.iter() {
        let new_tag = NewTag {
            name: tag_name.to_owned(),
        };

        let _ = tag::insert(&conn, &new_tag);
    }
}

pub fn sanitize_tags<'a>(tags: &'a str) -> Vec<String> {
    tags.split_whitespace()
        .map(|str| str.to_lowercase())
        .collect::<Vec<_>>()
}

pub fn rebuild(conn: &SqliteConnection) {
    use diesel::prelude::*;

    let limit = 250;
    let mut offset: i64 = 0;

    loop {
        let mut buffer: HashSet<String> = HashSet::new();

        let tag_strings: Vec<String> = uploads::table
            .select(uploads::tag_string)
            .filter(uploads::status.eq(UploadStatus::Completed))
            .order(uploads::id)
            .limit(limit)
            .offset(offset)
            .load::<String>(conn)
            .unwrap();

        let record_count = tag_strings.len() as i64;

        dedupe_tags(&tag_strings, &mut buffer);

        for tag_name in buffer.iter() {
            let new_tag = NewTag {
                name: tag_name.to_owned(),
            };

            let _ = tag::insert(&conn, &new_tag);
        }

        offset += record_count;

        debug!("[tag_service] rebuild limit={}, offset={}", limit, offset);

        if record_count < limit {
            debug!("[tag_service] rebuild finished!");
            break;
        }
    }
}

/// Recounts how many completed uploads use each tag, returning the tags whose count changed.
pub fn rebuild_tag_counts(conn: &SqliteConnection) -> Vec<Tag> {
    use crate::schema::tags;
    use diesel::prelude::*;
    use std::collections::HashMap;

    conn.transaction::<_, diesel::result::Error, _>(|| {
        let tag_strings: Vec<String> = uploads::table
            .select(uploads::tag_string)
            .filter(uploads::status.eq(UploadStatus::Completed))
            .load(conn)?;

        let mut true_counts: HashMap<&str, i32> = HashMap::new();

        for tag in tag_strings.iter().flat_map(|tag_string| tag_string.split_whitespace()) {
            *true_counts.entry(tag).or_insert(0) += 1;
        }

        let mut changed_tags = Vec::new();

        for mut tag in tags::table.load::<Tag>(conn)? {
            let true_count = true_counts.get(tag.name.as_str()).copied().unwrap_or(0);

            if tag.upload_count != true_count {
                diesel::update(tags::table.find(tag.id))
                    .set(tags::upload_count.eq(true_count))
                    .execute(conn)?;

                tag.upload_count = true_count;
                changed_tags.push(tag);
            }
        }

        Ok(changed_tags)
    })
    .unwrap_or_default()
}

fn dedupe_tags<'a>(tag_strings: &Vec<String>, buffer: &'a mut HashSet<String>) {
    for tag_string in tag_strings.iter() {
        let sanitized_tags = sanitize_tags(tag_string);

        for tag in sanitized_tags {
            buffer.insert(tag);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TagGroup {
    pub name: &'static str,
    pub tags: Vec<Tag>,
}

impl TagGroup {
    pub fn new(name: &'static str) -> TagGroup {
        TagGroup {
            name,
            tags: Vec::new(),
        }
    }
}

/// Groups tags into defined, common tag groups.
pub fn group_tags(tags: Vec<Tag>) -> (Vec<TagGroup>, Vec<Tag>) {
    let mut groups = vec![
        TagGroup::new("Communities"),
        TagGroup::new("Spinners"),
        TagGroup::new("Collaboration Video"),
        TagGroup::new("Promo Video"),
        TagGroup::new("Solo Video"),
        TagGroup::new("Editors"),
        TagGroup::new("Events"),
        TagGroup::new("Video Type"),
        TagGroup::new("Organizers"),
        TagGroup::new("Teams"),
    ];

    let remaining_tags = tags
        .into_iter()
        .filter(|tag| {
            let mut any_matches = false;

            if tag.name.starts_with("community") {
                groups[0].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("spinner") {
                groups[1].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("cv") {
                groups[2].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("pv") {
                groups[3].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("sv") {
                groups[4].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("editor") {
                groups[5].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("event") {
                groups[6].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("type") {
                groups[7].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("organizer") {
                groups[8].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("team") {
                groups[9].tags.push(tag.clone());
                any_matches = true;
            }

            return !any_matches;
        })
        .collect();

    (groups, remaining_tags)
}
