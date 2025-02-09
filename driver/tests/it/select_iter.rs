// Copyright 2023 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use databend_driver::DatabendConnection;
use tokio_stream::StreamExt;

use crate::common::DEFAULT_DSN;

fn prepare(name: &str) -> (DatabendConnection, String) {
    let dsn = option_env!("TEST_DATABEND_DSN").unwrap_or(DEFAULT_DSN);
    let table = format!("{}_{}", name, chrono::Utc::now().timestamp());
    (DatabendConnection::create(dsn).unwrap(), table)
}

#[tokio::test]
async fn select_iter() {
    let (conn, table) = prepare("select_iter");
    let sql_create = format!(
        "CREATE TABLE `{}` (
		i64 Int64,
		u64 UInt64,
		f64 Float64,
		s   String,
		s2  String,
		a16 Array(Int16),
		a8  Array(UInt8),
		d   Date,
		t   DateTime
    )",
        table
    );
    conn.exec(&sql_create).await.unwrap();
    let sql_insert = format!(
        "INSERT INTO `{}` VALUES
        (-1, 1, 1.0, '1', '1', [1], [10], '2011-03-06', '2011-03-06 06:20:00'),
        (-2, 2, 2.0, '2', '2', [2], [20], '2012-05-31', '2012-05-31 11:20:00'),
        (-3, 3, 3.0, '3', '2', [3], [30], '2016-04-04', '2016-04-04 11:30:00')",
        table
    );
    type RowResult = (
        i64,
        u64,
        f64,
        String,
        String,
        String,
        String,
        chrono::NaiveDate,
        chrono::NaiveDateTime,
    );
    let expected: Vec<RowResult> = vec![
        (
            -1,
            1,
            1.0,
            "1".into(),
            "1".into(),
            "[1]".into(),
            "[10]".into(),
            chrono::NaiveDate::from_ymd_opt(2011, 3, 6).unwrap(),
            chrono::DateTime::parse_from_rfc3339("2011-03-06T06:20:00Z")
                .unwrap()
                .naive_utc(),
        ),
        (
            -2,
            2,
            2.0,
            "2".into(),
            "2".into(),
            "[2]".into(),
            "[20]".into(),
            chrono::NaiveDate::from_ymd_opt(2012, 5, 31).unwrap(),
            chrono::DateTime::parse_from_rfc3339("2012-05-31T11:20:00Z")
                .unwrap()
                .naive_utc(),
        ),
        (
            -3,
            3,
            3.0,
            "3".into(),
            "2".into(),
            "[3]".into(),
            "[30]".into(),
            chrono::NaiveDate::from_ymd_opt(2016, 4, 4).unwrap(),
            chrono::DateTime::parse_from_rfc3339("2016-04-04T11:30:00Z")
                .unwrap()
                .naive_utc(),
        ),
    ];
    conn.exec(&sql_insert).await.unwrap();
    let sql_select = format!("SELECT * FROM `{}`", table);
    let mut rows = conn.query_iter(&sql_select).await.unwrap();
    let mut row_count = 0;
    while let Some(row) = rows.next().await {
        let v: RowResult = row.unwrap().try_into().unwrap();
        assert_eq!(v, expected[row_count]);
        row_count += 1;
    }
}
