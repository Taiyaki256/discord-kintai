use crate::database::models::{AttendanceRecord, RecordType};
use chrono::{DateTime, Utc, NaiveDate, NaiveTime, Timelike};
use anyhow::Result;

pub struct RecordValidator;

impl RecordValidator {
    /// 新しい記録が既存の記録と適切な順序になっているかチェック
    pub fn validate_record_order(
        existing_records: &[AttendanceRecord],
        new_record_type: RecordType,
        new_timestamp: DateTime<Utc>,
    ) -> Result<()> {
        // 時系列順にソート
        let mut sorted_records = existing_records.to_vec();
        sorted_records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // 新しい記録を挿入位置に配置して検証
        let mut all_records = Vec::new();
        let mut inserted = false;

        for record in sorted_records {
            if !inserted && record.timestamp > new_timestamp {
                // 新しい記録を挿入
                all_records.push(MockRecord {
                    timestamp: new_timestamp,
                    record_type: new_record_type,
                });
                inserted = true;
            }
            all_records.push(MockRecord {
                timestamp: record.timestamp,
                record_type: RecordType::from(record.record_type.clone()),
            });
        }

        // 最後に挿入されていない場合は末尾に追加
        if !inserted {
            all_records.push(MockRecord {
                timestamp: new_timestamp,
                record_type: new_record_type,
            });
        }

        // 順序を検証
        Self::validate_sequence(&all_records)
    }

    /// 記録シーケンスの妥当性をチェック
    fn validate_sequence(records: &[MockRecord]) -> Result<()> {
        let mut expect_start = true;
        let mut _session_count = 0;

        for (i, record) in records.iter().enumerate() {
            match record.record_type {
                RecordType::Start => {
                    if !expect_start {
                        return Err(anyhow::anyhow!(
                            "不正な順序: 位置{}で開始記録が連続しています（前回の終了記録がありません）",
                            i + 1
                        ));
                    }
                    expect_start = false;
                    _session_count += 1;
                }
                RecordType::End => {
                    if expect_start {
                        return Err(anyhow::anyhow!(
                            "不正な順序: 位置{}で終了記録がありますが、対応する開始記録がありません",
                            i + 1
                        ));
                    }
                    expect_start = true;
                }
            }
        }

        Ok(())
    }

    /// 同じ時刻の記録がないかチェック
    pub fn validate_no_duplicate_time(
        existing_records: &[AttendanceRecord],
        new_timestamp: DateTime<Utc>,
        exclude_record_id: Option<i64>,
    ) -> Result<()> {
        for record in existing_records {
            // 修正対象の記録は除外
            if let Some(exclude_id) = exclude_record_id {
                if record.id == exclude_id {
                    continue;
                }
            }

            if record.timestamp == new_timestamp {
                return Err(anyhow::anyhow!(
                    "同じ時刻の記録が既に存在します: {}",
                    new_timestamp.format("%H:%M")
                ));
            }
        }
        Ok(())
    }

    /// 時間の妥当性をチェック（未来時刻、過度に古い時刻など）
    pub fn validate_reasonable_time(
        new_time: NaiveTime,
        new_date: NaiveDate,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let now_jst = now.with_timezone(&jst_offset);
        let today_jst = now_jst.date_naive();

        // 未来の日付チェック
        if new_date > today_jst {
            return Err(anyhow::anyhow!(
                "未来の日付には記録できません"
            ));
        }

        // 今日の場合、未来の時刻チェック
        if new_date == today_jst {
            let current_time = now_jst.time();
            if new_time > current_time {
                return Err(anyhow::anyhow!(
                    "未来の時刻には記録できません"
                ));
            }
        }

        // 過度に古い記録のチェック（7日以上前）
        let days_ago = today_jst.signed_duration_since(new_date).num_days();
        if days_ago > 7 {
            return Err(anyhow::anyhow!(
                "7日以上前の記録は追加できません"
            ));
        }

        // 勤務時間として妥当な時刻かチェック（深夜2時〜朝5時は警告）
        if new_time.hour() >= 2 && new_time.hour() < 5 {
            return Err(anyhow::anyhow!(
                "深夜の時間帯です。本当に正しい時刻ですか？"
            ));
        }

        Ok(())
    }

    /// 包括的なバリデーション
    pub fn validate_new_record(
        existing_records: &[AttendanceRecord],
        new_record_type: RecordType,
        new_timestamp: DateTime<Utc>,
        new_date: NaiveDate,
        exclude_record_id: Option<i64>,
    ) -> Result<()> {
        let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let new_time_jst = new_timestamp.with_timezone(&jst_offset).time();

        // 1. 時間の妥当性チェック
        Self::validate_reasonable_time(new_time_jst, new_date)?;

        // 2. 重複時間チェック
        Self::validate_no_duplicate_time(existing_records, new_timestamp, exclude_record_id)?;

        // 3. 記録順序チェック
        Self::validate_record_order(existing_records, new_record_type, new_timestamp)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MockRecord {
    timestamp: DateTime<Utc>,
    record_type: RecordType,
}