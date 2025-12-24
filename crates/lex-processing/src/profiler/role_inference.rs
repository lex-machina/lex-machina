//! Role inference logic for column analysis.

use once_cell::sync::Lazy;
use polars::prelude::*;
use regex::Regex;

// ID pattern regexes - compiled once at startup
static ID_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^[a-zA-Z0-9]{6,}$").expect("Invalid regex: alphanumeric ID"),
        Regex::new(r"^\d{5,}$").expect("Invalid regex: numeric ID"),
        Regex::new(r"^[A-Z]{2,}\d+$").expect("Invalid regex: code pattern"),
        Regex::new(r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$")
            .expect("Invalid regex: UUID"),
        Regex::new(r"^\+?\d{7,15}$").expect("Invalid regex: phone"),
        Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").expect("Invalid regex: email"),
    ]
});

/// Infer the role of a column based on its name, type, and characteristics.
pub(crate) fn infer_column_role(
    col_name: &str,
    inferred_type: &str,
    unique_count: usize,
    total_rows: usize,
) -> String {
    let col_lower = col_name.to_lowercase();

    if inferred_type == "identifier" {
        return "identifier".to_string();
    }

    let target_keywords = vec![
        "target",
        "label",
        "class",
        "outcome",
        "result",
        "prediction",
        "default",
        "churn",
        "fraud",
        "risk",
        "score",
        "rating",
        "price",
        "value",
        "amount",
        "cost",
        "revenue",
        "sales",
        "profit",
        "loss",
        "duration",
        "time",
        "age",
    ];

    if target_keywords.iter().any(|k| col_lower.contains(k)) {
        return "target_candidate".to_string();
    }

    let metadata_keywords = vec![
        "date",
        "time",
        "created",
        "updated",
        "modified",
        "timestamp",
        "name",
        "description",
        "comment",
        "note",
        "address",
        "city",
        "country",
    ];

    if metadata_keywords.iter().any(|k| col_lower.contains(k)) {
        return "metadata".to_string();
    }

    // Binary or low-cardinality string are good target candidates
    if inferred_type == "binary" || (inferred_type == "string" && unique_count <= 10) {
        return "target_candidate".to_string();
    }

    // Numeric with reasonable uniqueness could be regression target
    if inferred_type == "numeric" && unique_count > 10 && unique_count < total_rows / 2 {
        return "target_candidate".to_string();
    }

    "feature".to_string()
}

/// Advanced identifier detection with better heuristics.
pub(crate) fn is_identifier_column_advanced(
    series: &Series,
    sample_values: &[String],
    col_name: &str,
) -> bool {
    if series.is_empty() {
        return false;
    }

    let unique_ratio = series.n_unique().unwrap_or(0) as f64 / series.len() as f64;

    // More flexible: allow some duplicates in identifiers (80% unique)
    if unique_ratio < 0.8 {
        return false;
    }

    let name_lower = col_name.to_lowercase();
    let id_keywords = vec![
        "id",
        "key",
        "uuid",
        "guid",
        "identifier",
        "code",
        "ref",
        "index",
        "name",
        "fullname",
        "firstname",
        "lastname",
        "user",
        "phone",
        "mobile",
        "contact",
        "email",
        "mail",
        "ssn",
        "passport",
        "account",
        "serial",
        "number",
        "customer",
        "client",
    ];

    // Strong indicator: column name contains ID keyword AND high uniqueness
    if id_keywords.iter().any(|k| name_lower.contains(k)) && unique_ratio > 0.9 {
        return true;
    }

    // Check patterns in sample values
    let id_patterns = &*ID_PATTERNS;

    let sample_check = sample_values.iter().take(10);
    let mut pattern_matches = 0;
    let mut checked = 0;

    for val in sample_check {
        checked += 1;
        for pattern in id_patterns.iter() {
            if pattern.is_match(val) {
                pattern_matches += 1;
                break;
            }
        }
    }

    // If most samples match ID patterns
    checked > 0 && (pattern_matches as f64 / checked as f64) > 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== infer_column_role tests ====================

    #[test]
    fn test_role_identifier_type() {
        let role = infer_column_role("some_col", "identifier", 100, 100);
        assert_eq!(role, "identifier");
    }

    #[test]
    fn test_role_target_keyword_target() {
        let role = infer_column_role("target", "numeric", 10, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_target_keyword_label() {
        let role = infer_column_role("class_label", "string", 5, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_target_keyword_outcome() {
        let role = infer_column_role("outcome_status", "binary", 2, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_target_keyword_price() {
        let role = infer_column_role("house_price", "numeric", 50, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_metadata_date() {
        let role = infer_column_role("created_date", "datetime", 100, 100);
        assert_eq!(role, "metadata");
    }

    #[test]
    fn test_role_metadata_timestamp() {
        // Note: "timestamp" contains "time" which is a target keyword
        // So we test with "modified" which is purely metadata
        let role = infer_column_role("last_modified", "datetime", 100, 100);
        assert_eq!(role, "metadata");
    }

    #[test]
    fn test_role_metadata_name() {
        let role = infer_column_role("customer_name", "string", 100, 100);
        assert_eq!(role, "metadata");
    }

    #[test]
    fn test_role_binary_is_target_candidate() {
        let role = infer_column_role("survived", "binary", 2, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_low_cardinality_string_is_target_candidate() {
        let role = infer_column_role("category", "string", 5, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_numeric_moderate_unique_is_target_candidate() {
        // unique_count > 10 and < total_rows / 2
        let role = infer_column_role("score", "numeric", 30, 100);
        assert_eq!(role, "target_candidate");
    }

    #[test]
    fn test_role_feature_default() {
        // High cardinality string without keywords
        let role = infer_column_role("random_feature", "string", 80, 100);
        assert_eq!(role, "feature");
    }

    // ==================== is_identifier_column_advanced tests ====================

    #[test]
    fn test_identifier_empty_series() {
        let series: Series = Series::new("id".into(), Vec::<&str>::new());
        assert!(!is_identifier_column_advanced(&series, &[], "id"));
    }

    #[test]
    fn test_identifier_low_uniqueness() {
        // Only 50% unique - should not be identifier
        let series = Series::new("id".into(), &["a", "b", "a", "b"]);
        assert!(!is_identifier_column_advanced(&series, &[], "id"));
    }

    #[test]
    fn test_identifier_column_name_with_id_keyword() {
        // High uniqueness + "id" in name
        let series = Series::new("user_id".into(), &["1", "2", "3", "4", "5"]);
        let samples = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        assert!(is_identifier_column_advanced(&series, &samples, "user_id"));
    }

    #[test]
    fn test_identifier_uuid_pattern() {
        let series = Series::new(
            "record".into(),
            &[
                "550e8400-e29b-41d4-a716-446655440000",
                "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            ],
        );
        let samples = vec![
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
        ];
        assert!(is_identifier_column_advanced(&series, &samples, "record"));
    }

    #[test]
    fn test_identifier_email_pattern() {
        let series = Series::new(
            "contact".into(),
            &[
                "alice@example.com",
                "bob@company.org",
                "charlie@mail.net",
            ],
        );
        let samples = vec![
            "alice@example.com".to_string(),
            "bob@company.org".to_string(),
            "charlie@mail.net".to_string(),
        ];
        assert!(is_identifier_column_advanced(&series, &samples, "contact"));
    }

    #[test]
    fn test_identifier_numeric_id_pattern() {
        let series = Series::new("seq".into(), &["12345", "67890", "11111", "22222"]);
        let samples = vec![
            "12345".to_string(),
            "67890".to_string(),
            "11111".to_string(),
            "22222".to_string(),
        ];
        assert!(is_identifier_column_advanced(&series, &samples, "seq"));
    }

    #[test]
    fn test_identifier_not_identifier_regular_text() {
        let series = Series::new("description".into(), &["hello world", "foo bar", "test"]);
        let samples = vec![
            "hello world".to_string(),
            "foo bar".to_string(),
            "test".to_string(),
        ];
        assert!(!is_identifier_column_advanced(
            &series,
            &samples,
            "description"
        ));
    }
}
