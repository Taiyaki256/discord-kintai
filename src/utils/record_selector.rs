use crate::database::models::AttendanceRecord;
use crate::utils::time::format_time_jst;
use poise::serenity_prelude as serenity;

pub struct RecordSelector {
    records: Vec<AttendanceRecord>,
}

impl RecordSelector {
    pub fn new(mut records: Vec<AttendanceRecord>) -> Self {
        // Sort by timestamp for chronological order
        records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Self { records }
    }

    pub fn create_select_menu(
        &self,
        custom_id: &str,
        placeholder: &str,
    ) -> Option<serenity::CreateSelectMenu> {
        if self.records.is_empty() {
            return None;
        }

        let mut options = Vec::new();

        for record in &self.records {
            let time_str = format_time_jst(record.timestamp);
            let type_str = match record.record_type.as_str() {
                "start" => "開始",
                "end" => "終了",
                _ => "不明",
            };

            let modified_indicator = if record.is_modified {
                " (修正済み)"
            } else {
                ""
            };

            let label = format!("{} {}{}", time_str, type_str, modified_indicator);
            let description = if record.is_modified {
                if let Some(original) = record.original_timestamp {
                    format!("元の時間: {}", format_time_jst(original))
                } else {
                    "修正済みの記録".to_string()
                }
            } else {
                format!("記録ID: {}", record.id)
            };

            options.push(
                serenity::CreateSelectMenuOption::new(label, record.id.to_string())
                    .description(description),
            );
        }

        // Discord select menu limit is 25 options
        if options.len() > 25 {
            options.truncate(25);
        }

        Some(
            serenity::CreateSelectMenu::new(
                custom_id,
                serenity::CreateSelectMenuKind::String { options },
            )
            .placeholder(placeholder),
        )
    }

    pub fn create_delete_select_menu(&self, custom_id: &str) -> Option<serenity::CreateSelectMenu> {
        if self.records.is_empty() {
            return None;
        }

        let mut options = Vec::new();

        // Add individual record options
        for record in &self.records {
            let time_str = format_time_jst(record.timestamp);
            let type_str = match record.record_type.as_str() {
                "start" => "開始",
                "end" => "終了",
                _ => "不明",
            };

            let label = format!("{} {}", time_str, type_str);
            options.push(
                serenity::CreateSelectMenuOption::new(label, record.id.to_string())
                    .description(format!("記録ID: {}", record.id)),
            );
        }

        // Add "delete all" option if there are multiple records
        if self.records.len() > 1 {
            options.push(
                serenity::CreateSelectMenuOption::new("🗑️ 全て削除", "delete_all")
                    .description("当日のすべての記録を削除します"),
            );
        }

        // Discord select menu limit is 25 options
        if options.len() > 25 {
            options.truncate(25);
        }

        Some(
            serenity::CreateSelectMenu::new(
                custom_id,
                serenity::CreateSelectMenuKind::String { options },
            )
            .placeholder("削除する記録を選択してください"),
        )
    }

    pub fn get_record_by_id(&self, id: i64) -> Option<&AttendanceRecord> {
        self.records.iter().find(|record| record.id == id)
    }

    pub fn get_all_record_ids(&self) -> Vec<i64> {
        self.records.iter().map(|record| record.id).collect()
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
