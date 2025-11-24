use toonconv::conversion::{QuoteStrategy, DelimiterType};

#[test]
fn test_quote_strategy_smart() {
    // Empty string
    assert!(QuoteStrategy::Smart.should_quote("", DelimiterType::Comma));

    // Keywords
    assert!(QuoteStrategy::Smart.should_quote("null", DelimiterType::Comma));
    assert!(QuoteStrategy::Smart.should_quote("true", DelimiterType::Comma));

    // Numeric string
    assert!(QuoteStrategy::Smart.should_quote("42", DelimiterType::Comma));

    // Leading/trailing whitespace
    assert!(QuoteStrategy::Smart.should_quote(" hello", DelimiterType::Comma));
    assert!(QuoteStrategy::Smart.should_quote("hello ", DelimiterType::Comma));

    // Structural characters
    assert!(QuoteStrategy::Smart.should_quote("a:b", DelimiterType::Comma));
    assert!(QuoteStrategy::Smart.should_quote("str{", DelimiterType::Comma));

    // Delimiter chars
    assert!(QuoteStrategy::Smart.should_quote("a,b", DelimiterType::Comma));
    assert!(!QuoteStrategy::Smart.should_quote("a,b", DelimiterType::Tab));

    // Normal string doesn't require quotes
    assert!(!QuoteStrategy::Smart.should_quote("hello", DelimiterType::Comma));
}
