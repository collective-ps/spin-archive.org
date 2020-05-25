use std::collections::HashSet;

use diesel::PgConnection;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::models::tag::{self, NewTag, Tag};
use crate::models::upload::UploadStatus;
use crate::schema::uploads;

pub use crate::models::tag::by_names;

pub fn create_from_tag_string(conn: &PgConnection, tag_string: &str) {
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

pub fn rebuild(conn: &PgConnection) {
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

pub fn rebuild_tag_counts(conn: &PgConnection) -> Vec<Tag> {
    use diesel::prelude::*;

    let mut tags = Vec::with_capacity(1000);

    let mut updated_tags = diesel::sql_query(
        "UPDATE tags SET upload_count = true_count FROM (
        SELECT tag, COUNT(*) AS true_count
        FROM uploads,
        unnest(string_to_array(tag_string, ' ')) AS tag
        WHERE uploads.status = 2
        GROUP BY tag
      ) true_counts WHERE tags.name = tag AND tags.upload_count != true_count RETURNING tags.*",
    )
    .load::<Tag>(conn)
    .unwrap_or_default();

    let mut removed_tags = diesel::sql_query(
        "UPDATE tags SET upload_count = 0 WHERE upload_count != 0 AND name NOT IN (
        SELECT DISTINCT tag
        FROM uploads, unnest(string_to_array(tag_string, ' ')) AS tag
        GROUP BY tag
      ) RETURNING tags.*",
    )
    .load::<Tag>(conn)
    .unwrap_or_default();

    tags.append(&mut updated_tags);
    tags.append(&mut removed_tags);
    tags
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
    name: &'static str,
    tags: Vec<Tag>,
}

impl TagGroup {
    pub fn new(name: &'static str) -> TagGroup {
        TagGroup {
            name,
            tags: Vec::new(),
        }
    }
}

pub fn group_tags(tags: Vec<Tag>) -> (Vec<TagGroup>, Vec<Tag>) {
    let mut groups = vec![
        TagGroup::new("cv"),
        TagGroup::new("spinner"),
        TagGroup::new("type"),
        TagGroup::new("community")
    ];

    let remaining_tags = tags
        .into_iter()
        .filter(|tag| {
            let mut any_matches = false;

            if tag.name.starts_with("cv:") {
                groups[0].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("spinner:") {
                groups[1].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("type:") {
                groups[2].tags.push(tag.clone());
                any_matches = true;
            }

            if tag.name.starts_with("community:") {
                groups[3].tags.push(tag.clone());
                any_matches = true;
            }

            return !any_matches;
        })
        .collect();

    (groups, remaining_tags)
}
