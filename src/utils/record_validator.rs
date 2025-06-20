use crate::database::models::{AttendanceRecord, RecordType};
use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

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

        // 新しい記録を適切な位置に挿入して検証
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
    /// 複数の開始・終了記録を許可する柔軟なバリデーション
    fn validate_sequence(records: &[MockRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(()); // 空の記録は有効
        }

        // 連続する同じタイプの記録をチェック
        let mut last_record_type: Option<RecordType> = None;
        let mut consecutive_count = 0;

        for (i, record) in records.iter().enumerate() {
            match last_record_type {
                Some(last_type) if last_type == record.record_type => {
                    consecutive_count += 1;
                    // 同じタイプが3回以上連続する場合は警告
                    if consecutive_count >= 3 {
                        return Err(anyhow::anyhow!(
                            "不正な順序: 位置{}で{}記録が{}回連続しています",
                            i + 1,
                            if record.record_type == RecordType::Start {
                                "開始"
                            } else {
                                "終了"
                            },
                            consecutive_count + 1
                        ));
                    }
                }
                _ => {
                    consecutive_count = 0;
                }
            }
            last_record_type = Some(record.record_type);
        }

        // 最初の記録が終了記録の場合は警告
        if let Some(first_record) = records.first() {
            if first_record.record_type == RecordType::End {
                return Err(anyhow::anyhow!(
                    "不正な順序: 開始記録なしに終了記録があります"
                ));
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
    pub fn validate_reasonable_time(new_time: NaiveTime, new_date: NaiveDate) -> Result<()> {
        let now = chrono::Utc::now();
        let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let now_jst = now.with_timezone(&jst_offset);
        let today_jst = now_jst.date_naive();

        // 未来の日付チェック
        if new_date > today_jst {
            return Err(anyhow::anyhow!("未来の日付には記録できません"));
        }

        // 今日の場合、未来の時刻チェック
        if new_date == today_jst {
            let current_time = now_jst.time();
            if new_time > current_time {
                return Err(anyhow::anyhow!("未来の時刻には記録できません"));
            }
        }

        // 過度に古い記録のチェック（7日以上前）
        let days_ago = today_jst.signed_duration_since(new_date).num_days();
        if days_ago > 7 {
            return Err(anyhow::anyhow!("7日以上前の記録は追加できません"));
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
