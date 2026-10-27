use std::time::Duration;

pub(crate) fn stream_interrupted(error: &str) -> bool {
    matches!(
        error,
        "Connection interrupted before the response finished."
            | "Connection ended before the tool response finished."
            | "Provider returned a malformed stream."
    ) || error.starts_with("Model provider ended an unfinished response")
        || error.starts_with("Model stream stalled without new response data")
        || error.starts_with("Model connection stalled")
}

/// Only adapter-owned generation errors are eligible. This is not tool retry.
pub(crate) fn delay(error: &str) -> Option<Duration> {
    if error == "Model rate limit reached. Try again later." {
        return Some(Duration::from_secs(5));
    }
    let status = error
        .strip_prefix("Tool request failed (HTTP ")
        .and_then(|s| s.split_once(')'))
        .and_then(|(s, _)| s.parse::<u16>().ok());
    if status.is_some_and(|s| matches!(s, 408 | 500 | 502 | 503 | 504))
        || matches!(
            error,
            "Could not reach the model. Check the endpoint and whether the server is running."
                | "Model request timed out. Adjust the request timeout or try again."
                | "Connection interrupted before the response finished."
                | "Connection ended before the tool response finished."
                | "Connection ended before the response finished."
                | "Provider returned a malformed stream."
                | "Provider reported an error while streaming."
                | "Model returned an invalid streamed tool response."
                | "Model returned an invalid tool response."
                | "Model returned incomplete or invalid tool arguments."
        )
        || error.starts_with("Model provider ended an unfinished response")
        || error.starts_with("Model provider returned inconsistent stream completion")
        || error.starts_with("Model stream stalled without new response data")
        || error.starts_with("Model connection stalled")
    {
        Some(Duration::from_secs(1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_generation_failures_retry() {
        assert!(stream_interrupted(
            "Model provider ended an unfinished response after 30s (last activity: reasoning)."
        ));
        assert!(!stream_interrupted(
            "Tool request failed (HTTP 503). Fixture"
        ));
        assert!(!stream_interrupted(
            "Model returned incomplete or invalid tool arguments."
        ));
        assert!(!stream_interrupted(
            "Model access denied. Check your API key and permissions."
        ));
        assert_eq!(
            delay("Model rate limit reached. Try again later."),
            Some(Duration::from_secs(5))
        );
        for error in [
            "Connection ended before the tool response finished.",
            "Model returned incomplete or invalid tool arguments.",
            "Model request timed out. Adjust the request timeout or try again.",
        ] {
            assert_eq!(delay(error), Some(Duration::from_secs(1)));
        }
        for status in [408, 500, 502, 503, 504] {
            assert!(delay(&format!("Tool request failed (HTTP {status}). Fixture")).is_some());
        }
        for error in [
            "Model access denied. Check your API key and permissions.",
            "Tool request failed (HTTP 400). Fixture",
            "Tool request failed (HTTP 501). Fixture",
            "Response stopped.",
            "Conversation window closed.",
            "Model context limit reached.",
            "Model reused a tool call ID.",
            "Streamed tool field exceeds its byte limit.",
            "Unknown plugin error",
            "Tool output failed",
        ] {
            assert!(delay(error).is_none(), "{error}");
        }
    }
}
