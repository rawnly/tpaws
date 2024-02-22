use color_eyre::{eyre::OptionExt, Result};
use regex::Regex;

pub(crate) fn branch_to_title(branch: String) -> String {
    let re = Regex::new(r#"\d+"#).unwrap();

    let sanitized_branch_title = re
        .replace(branch.split('/').last().unwrap_or(&branch), "")
        .replace('_', " ")
        .trim()
        .to_string();

    let (first, rest) = sanitized_branch_title.split_at(1);

    // capitalize
    format!("{}{rest}", first.to_uppercase())
}

pub(crate) fn get_ticket_id_from_branch(branch: String) -> Option<String> {
    let re = Regex::new(r#"\w+\/(\d+)_.*"#).unwrap();
    let captures = re.captures(&branch)?;

    captures.get(1).map(|s| s.as_str().to_string())
}

pub(crate) async fn grab_title(title: Option<String>, branch: String) -> Result<String> {
    match title {
        Some(title) => Ok(title),
        None => {
            if !target_process::has_token() {
                return Ok(branch_to_title(branch));
            }

            let id =
                get_ticket_id_from_branch(branch).ok_or_eyre("failed to retrive user_story ID")?;

            let assignable = target_process::get_assignable(&id).await?;

            Ok(assignable.name)
        }
    }
}

#[macro_export]
macro_rules! print_dbg {
    ( $( $x:expr ),* ) => {
        $(
            if cfg!(debug_assertions) {
                dbg!($x);
            }
        )*
    };
}

mod test {

    #[test]
    fn branch_to_title() {
        let string = "feature/115068_translate_report_type_payout_transactions".to_string();
        let output = super::branch_to_title(string);

        assert_eq!(
            output,
            "Translate report type payout transactions".to_string()
        )
    }

    #[test]
    fn extract_assignable_id() {
        let string = "feature/115068_translate_report_type_payout_transactions".to_string();

        let output = super::get_ticket_id_from_branch(string);

        assert_eq!(output, Some("115068".into()))
    }

    #[tokio::test]
    async fn grab_title_should_not_execute_async_code_if_title_is_given() {
        let data = super::grab_title(Some("demo".to_string()), "feature/120890_abc".to_string())
            .await
            .unwrap();

        assert_eq!("demo".to_string(), data);
    }
}
