use crate::LevelTrait;

/// The probability that at least one letter will fall after finding the first word
pub fn calculate_falling_probability_1(level: &impl LevelTrait) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0.0f32;
    for word_index in 0..(level.words().len()) {
        let unneeded = level.calculate_unneeded_tiles(Default::default(), |wi| wi == word_index);

        if !unneeded.is_empty() {
            total += 1.0;
        }
        count += 1.0;
    }

    total /= count;
    total
}

/// The probability that at least one additional letter will fall after finding the first word
pub fn calculate_falling_probability_2(level: &impl LevelTrait) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0.0f32;
    for word_index in 0..(level.words().len()) {
        let unneeded_after_1 =
            level.calculate_unneeded_tiles(Default::default(), |wi| wi == word_index);

        for word_index2 in 0..(level.words().len()) {
            if word_index2 != word_index {
                let unneeded_after_2 = level.calculate_unneeded_tiles(unneeded_after_1, |wi| {
                    wi == word_index || wi == word_index2
                });
                if unneeded_after_2 != unneeded_after_1 {
                    total += 1.0;
                }

                count += 1.0;
            }
        }
    }

    total /= count;
    total
}

/// The probability that at least one additional letter will fall after finding the first word
pub fn calculate_cumulative_falling_probability_2(level: &impl LevelTrait) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0.0f32;
    let word_count_sub_1 = (level.words().len() - 1) as f32;
    for word_index in 0..(level.words().len()) {
        let unneeded_after_1 =
            level.calculate_unneeded_tiles(Default::default(), |wi| wi == word_index);

        if !unneeded_after_1.is_empty() {
            total += word_count_sub_1;
            count += word_count_sub_1;
        } else {
            for word_index2 in 0..(level.words().len()) {
                if word_index2 != word_index {
                    let unneeded_after_2 = level.calculate_unneeded_tiles(unneeded_after_1, |wi| {
                        wi == word_index || wi == word_index2
                    });
                    if !unneeded_after_2.is_empty() {
                        total += 1.0;
                    }

                    count += 1.0;
                }
            }
        }
    }

    total /= count;
    total
}
