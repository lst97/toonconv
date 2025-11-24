use crate::conversion::{ConversionConfig, ConversionResult};
use crate::parser::JsonSource;
use crate::conversion::engine::ConversionEngine;

/// Batch convert multiple JsonSource inputs. Optionally continue on errors.
pub fn convert_batch_sources(sources: Vec<JsonSource>, config: &ConversionConfig, continue_on_error: bool) -> ConversionResult<Vec<(JsonSource, crate::conversion::ToonData)>> {
    let engine = ConversionEngine::new(config.clone());
    let mut results = Vec::new();

    for src in sources {
        match engine.convert_from_source(&src) {
            Ok(toon) => results.push((src, toon)),
            Err(e) => {
                if continue_on_error {
                    eprintln!("✗ Error converting source: {}", e.user_message());
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(results)
}
