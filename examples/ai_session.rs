//! AI session example showing streaming conversations and AI operations.

use futures::StreamExt;
use oramacore_client::{
    collection::{CollectionManager, CollectionManagerConfig, NlpSearchParams},
    error::Result,
    stream_manager::{AnswerConfig, CreateAiSessionConfig},
    types::{LlmConfig, LlmProvider, Message, Role},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the collection manager
    let config = CollectionManagerConfig::new("your-collection-id", "your-api-key");
    let client = CollectionManager::new(config).await?;

    // Example 1: NLP Search
    println!("=== NLP Search ===");
    let nlp_params = NlpSearchParams {
        query: "What are the benefits of machine learning?".to_string(),
        llm_config: Some(LlmConfig {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
        }),
        user_id: Some("user-123".to_string()),
    };

    let nlp_results = client.ai.nlp_search::<Document>(nlp_params).await?;
    println!("NLP search found {} results", nlp_results.len());

    for (i, result) in nlp_results.iter().enumerate() {
        println!("{}. Generated query: {:?}", i + 1, result.generated_query);
        println!("   Found {} results", result.results.len());
    }

    // Example 2: Create AI Session
    println!("\n=== Creating AI Session ===");
    let initial_messages = vec![Message {
        role: Role::System,
        content: "You are a helpful AI assistant specializing in technology and science."
            .to_string(),
    }];

    let _session_config = CreateAiSessionConfig::new()
        .with_llm_config(LlmConfig {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
        })
        .with_initial_messages(initial_messages);

    // Use the collection manager's method to create the session
    let ai_session = client.ai.create_ai_session().await?;
    println!("AI session created with ID: {}", ai_session.session_id());

    // Example 3: Simple AI Answer
    println!("\n=== Simple AI Answer ===");
    let answer_config = AnswerConfig::new("Explain quantum computing in simple terms")
        .with_max_documents(5)
        .with_min_similarity(0.8);

    let answer = ai_session.answer(answer_config).await?;
    println!("AI Response: {}", answer);

    // Example 4: Streaming AI Answer
    println!("\n=== Streaming AI Answer ===");
    let stream_config =
        AnswerConfig::new("What are the latest developments in artificial intelligence?")
            .with_visitor_id("demo-user".to_string())
            .with_max_documents(3);

    let mut answer_stream = ai_session.answer_stream(stream_config).await?;
    println!("Streaming response:");
    print!("AI: ");

    while let Some(chunk_result) = answer_stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                match chunk {
                    oramacore_client::stream_manager::StreamChunk::Content(content) => {
                        print!("{}", content);
                        // Flush stdout to see streaming in real-time
                        use std::io::{self, Write};
                        io::stdout().flush().unwrap();
                    },
                    oramacore_client::stream_manager::StreamChunk::StatusUpdate(status) => {
                        println!("\n[Status: {}]", status);
                    },
                    oramacore_client::stream_manager::StreamChunk::Done => {
                        println!("\n[Stream completed]");
                        break;
                    },
                    _ => {} // Ignore other chunk types for this demo
                }
            }
            Err(e) => {
                eprintln!("\nStream error: {}", e);
                break;
            }
        }
    }
    println!(); // New line after streaming

    // Example 5: Multiple conversation turns
    println!("\n=== Multi-turn Conversation ===");
    let questions = vec![
        "What is machine learning?",
        "How does it differ from traditional programming?",
        "Can you give me a practical example?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n--- Turn {} ---", i + 1);
        println!("User: {}", question);

        let turn_config = AnswerConfig::new(*question).with_visitor_id("demo-user".to_string());

        let response = ai_session.answer(turn_config).await?;
        println!("AI: {}", response);
    }

    // Example 6: Regenerate last response
    println!("\n=== Regenerating Last Response ===");
    let regenerated = ai_session.regenerate_last(false).await?;
    println!("Regenerated response: {}", regenerated);

    // Example 7: Get conversation history
    println!("\n=== Conversation History ===");
    let messages = ai_session.get_messages().await;
    println!("Conversation has {} messages:", messages.len());

    for (i, message) in messages.iter().enumerate() {
        let role = match message.role {
            Role::System => "System",
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        println!(
            "{}. {}: {}",
            i + 1,
            role,
            if message.content.len() > 100 {
                format!("{}...", &message.content[..100])
            } else {
                message.content.clone()
            }
        );
    }

    // Example 8: Get session state
    println!("\n=== Session State ===");
    let state = ai_session.get_state().await;
    println!("Session has {} interactions:", state.len());

    for (i, interaction) in state.iter().enumerate() {
        println!("{}. Query: {}", i + 1, interaction.query);
        println!("   Response length: {} chars", interaction.response.len());
        println!(
            "   Loading: {}, Error: {}",
            interaction.loading, interaction.error
        );
    }

    // Example 9: Clear session
    println!("\n=== Clearing Session ===");
    ai_session.clear_session().await;
    println!("Session cleared");

    let final_messages = ai_session.get_messages().await;
    println!("Messages after clearing: {}", final_messages.len());

    Ok(())
}
