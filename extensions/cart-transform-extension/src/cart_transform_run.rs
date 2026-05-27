use super::schema;
use serde::Deserialize;
use shopify_function::prelude::*;
use shopify_function::Result;

// Type definition for Shopify's specialized Decimal wrapper
use shopify_function::prelude::Decimal;

#[derive(Deserialize, Debug)]
struct FlowerArrangement {
    flower: Vec<String>,
    arrangement: String,
    color: String,
}

#[shopify_function]
fn cart_transform_run(
    _input: schema::cart_transform_run::CartTransformRunInput,
) -> Result<schema::CartTransformRunResult> {
    // 1. Filter out lines that don't possess a valid configuration value
    let lines: Vec<&schema::cart_transform_run::cart_transform_run_input::cart::Lines> = _input
        .cart()
        .lines()
        .iter()
        .filter(|line| line.config().and_then(|attr| attr.value()).is_some())
        .collect();

    if lines.is_empty() {
        return Ok(schema::CartTransformRunResult { operations: vec![] });
    }

    // 2. Map through matched cart lines to generate transformation operations
    let operations: Vec<schema::Operation> = lines
        .iter()
        .filter_map(|line| {
            // Safe extraction of the JSON configuration string
            let config_str = line.config().and_then(|attr| attr.value())?;

            // 3. Deserialize the JSON directly inside the iterator using serde_json
            let arrangement_data: FlowerArrangement = match serde_json::from_str(config_str) {
                Ok(parsed) => parsed,
                Err(err) => {
                    log!("Failed to parse configuration payload: {}", err);
                    return None; // Skip this line item if the JSON string structure is invalid
                }
            };

            // 4. Calculate Custom Title Layout ("rose, lily | bouquet | rose")
            // Joining an array of strings by a comma automatically adds the correct spacing
            let formatted_flowers = arrangement_data.flower.join(", ");
            let custom_title = format!(
                "{} | {} | {}",
                formatted_flowers, arrangement_data.arrangement, arrangement_data.color
            );

            // 5. Run Price Calculation Logic
            let mut total_price: f64 = 0.0;

            // Increment $20 per flower entry in the config array
            for _ in &arrangement_data.flower {
                total_price += 20.0;
            }

            // Apply base structural pricing modifiers based on arrangement variant
            match arrangement_data.arrangement.as_str() {
                "bouquet" => total_price += 30.0,
                "basket"  => total_price += 25.0,
                "vase"    => total_price += 20.0,
                "box"     => total_price += 10.0,
                _         => total_price += 0.0, // Fallback safety clause
            }

            // 6. Construct the Shopify Line Update Operation
            Some(schema::Operation::LineUpdate(schema::LineUpdateOperation {
                cart_line_id: line.id().to_string(),
                image: None,
                title: Some(custom_title),
                price: Some(schema::LineUpdateOperationPriceAdjustment {
                    adjustment: schema::LineUpdateOperationPriceAdjustmentValue::FixedPricePerUnit(
                        schema::LineUpdateOperationFixedPricePerUnitAdjustment {
                            amount: Decimal(total_price),
                        },
                    ),
                }),
            }))
        })
        .collect();

    Ok(schema::CartTransformRunResult { operations })
}