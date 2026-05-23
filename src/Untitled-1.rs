HashMap<u64, HashMap<u64, Vec<String>>>

pub async fn ask_ai(
    question: &str,
    model: &str,
    channel_id: u64,
    user_id: u64,
    username: &str,
)

// inside the ask_ai function from linr 49

let previous_history;

{
    let mut memory =
        CHANNEL_HISTORY.lock().await;

    // Get channel memory
    let channel_memory =
        memory
        .entry(channel_id)
        .or_insert(HashMap::new());

    // Get user memory inside channel
    let user_history =
        channel_memory
        .entry(user_id)
        .or_insert(Vec::new());

    user_history.push(
        format!(
            "{}: {}",
            username,
            question
        )
    );

    if user_history.len() > 10 {
        user_history.remove(0);
    }

    previous_history =
        user_history.join("\n");
}

// add this before the save_memory() call

{
    let mut memory =
        CHANNEL_HISTORY.lock().await;

    let channel_memory =
        memory
        .entry(channel_id)
        .or_insert(HashMap::new());

    let user_history =
        channel_memory
        .entry(user_id)
        .or_insert(Vec::new());

    user_history.push(
        format!(
            "AI: {}",
            answer
        )
    );
}

save_memory().await;