mod chat;
mod config;
mod openrouter;
mod system_prompt;
mod telegram;
mod ui;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let config = config::Config::load()?;

    let system_prompt =
        system_prompt::build_system_prompt(config.child_name.as_deref());

    let mut chat = chat::ChatHistory::new(system_prompt, config.max_history);

    let openrouter = openrouter::OpenRouterClient::new(
        config.openrouter_api_key,
        config.openrouter_model,
    );

    let telegram = telegram::TelegramNotifier::new(
        config.telegram_bot_token,
        config.telegram_chat_id,
    );

    let child_name = config.child_name;

    ui::print_welcome(child_name.as_deref());

    let mut editor = DefaultEditor::new()?;
    let prompt = ui::prompt_string();
    let mut telegram_tasks = Vec::new();

    loop {
        let input = editor.readline(&prompt);

        match input {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                if matches!(
                    trimmed.to_lowercase().as_str(),
                    "quit" | "exit" | "bye"
                ) {
                    ui::print_goodbye(child_name.as_deref());
                    break;
                }

                let _ = editor.add_history_entry(trimmed);

                chat.add_user_message(trimmed);

                ui::print_thinking();

                let api_messages = chat.build_api_messages();

                let mut first_token = true;
                let mut wrapper = ui::WordWrapper::new(4); // "AI> " = 4 cols

                let result = openrouter
                    .stream_chat(&api_messages, |token| {
                        if first_token {
                            ui::clear_thinking();
                            ui::print_ai_prefix();
                            first_token = false;
                        }
                        wrapper.push(token);
                    })
                    .await;

                wrapper.finish();

                match result {
                    Ok(response) => {
                        if first_token {
                            // No tokens were received
                            ui::clear_thinking();
                            ui::print_ai_prefix();
                            println!("Hmm, I didn't get a response. Try asking again!");
                        } else {
                            ui::print_ai_done();
                        }

                        chat.add_assistant_message(&response);

                        telegram_tasks.push(telegram.notify(trimmed, &response));
                    }
                    Err(e) => {
                        if !first_token {
                            println!();
                        } else {
                            ui::clear_thinking();
                        }
                        eprintln!("OpenRouter error: {e}");
                        ui::print_error(
                            "Something went wrong. Try asking again!",
                        );
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C
                ui::print_goodbye(child_name.as_deref());
                break;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D
                ui::print_goodbye(child_name.as_deref());
                break;
            }
            Err(e) => {
                eprintln!("Input error: {e}");
                ui::print_error("Something went wrong with input. Try again!");
            }
        }
    }

    // Wait for all background Telegram tasks to complete before exiting.
    for task in telegram_tasks {
        let _ = task.await;
    }

    Ok(())
}
