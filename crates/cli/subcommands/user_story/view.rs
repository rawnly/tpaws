use color_eyre::eyre::Result;
use commands::spawn_command;
use mdka::from_html;

use crate::utils;

pub async fn view(id_or_url: Option<String>, json: bool, web: bool) -> Result<()> {
    let id = utils::extract_id(id_or_url).await?;
    let assignable = target_process::get_assignable(id).await?;

    if web {
        spawn_command!("open", assignable.get_link())?;
        return Ok(());
    }

    if json {
        let json_string = serde_json::to_string_pretty(&assignable)?;
        println!("{}", json_string);

        return Ok(());
    }

    println!("{}", assignable.name);
    println!("===================");
    println!();

    match assignable.description {
        Some(description) => print_ticket_body(description),
        None => println!("no description provided."),
    };

    println!();

    Ok(())
}

fn print_ticket_body(description: String) {
    if !description.starts_with("<!--markdown-->") {
        let description = from_html(&description);

        termimad::print_text(&description);
        return;
    }

    termimad::print_text(&description.replace("<!--markdown-->", ""));
}
