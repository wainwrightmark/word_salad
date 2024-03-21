use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use ws_core::{layout::entities::SelfieMode, DesignedLevel, LayoutSizing, LayoutStructure};

pub fn do_word_layout() {
    let folder = std::fs::read_dir("grids").unwrap();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {bar} {pos:2}/{len:2}").unwrap())
        .with_message("Grid files");

    let mut longest_line: (isize, Option<DesignedLevel>) = Default::default();
    let mut shortest_line: (isize, Option<DesignedLevel>) = (isize::MAX, None);
    let mut most_lines: (usize, Option<DesignedLevel>) = Default::default();
    let mut max_diff: (isize, Option<DesignedLevel>) = Default::default();

    let sizing = LayoutSizing::default();
    let selfie_mode = SelfieMode {
        is_selfie_mode: false,
    };

    for path in paths.iter() {
        let grids_path = path.as_ref().unwrap().path();
        let grid_file_text = std::fs::read_to_string(grids_path.clone()).unwrap();

        for line in grid_file_text.lines() {
            let level = DesignedLevel::from_tsv_line(line).unwrap();

            struct GroupData {
                rightmost: isize,
            }

            let context = &(
                level.words.as_slice(),
                (selfie_mode, ws_core::Insets::default()),
            );
            let data: Vec<GroupData> =
                ws_core::layout::entities::layout_word_tile::LayoutWordTile::iter_all(context)
                    .group_by(|tile| tile.location(context, &sizing).y)
                    .into_iter()
                    .map(|(_, group)| GroupData {
                        rightmost: group
                            .map(|t| t.rect(context, &sizing).centre_right().x.ceil() as isize)
                            .max()
                            .unwrap_or_default(),
                    })
                    .collect();

            if data.len() > most_lines.0 {
                most_lines.0 = data.len();
                most_lines.1 = Some(level.clone());
            }

            let longest = data.iter().map(|x| x.rightmost).max().unwrap_or_default();
            let shortest = data.iter().map(|x| x.rightmost).min().unwrap_or_default();
            let diff = longest - shortest;

            if diff > max_diff.0 {
                max_diff.0 = diff;
                max_diff.1 = Some(level.clone());
            }

            for d in data {
                if d.rightmost > longest_line.0 {
                    longest_line.0 = d.rightmost;
                    longest_line.1 = Some(level.clone());
                }

                if d.rightmost < shortest_line.0 {
                    shortest_line.0 = d.rightmost;
                    shortest_line.1 = Some(level.clone());
                }
            }
        }

        pb.inc(1);
    }

    if let Some(l) = longest_line.1 {
        info!("Longest Line: {}. {l}", longest_line.0);
    }

    if let Some(l) = shortest_line.1 {
        info!("Shortest Line: {}. {l}", shortest_line.0);
    }

    if let Some(l) = most_lines.1 {
        info!("Most lines {}. {l}", most_lines.0);
    }

    if let Some(l) = max_diff.1 {
        info!("Max Diff {}. {l}", max_diff.0);
    }
}
