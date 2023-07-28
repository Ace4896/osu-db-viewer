//! Models for the main `osu.db` database file, which contains information on installed beatmaps.

use nom::{
    bytes::complete::tag,
    combinator::{cond, map},
    multi::count,
    number::complete::{le_f32, le_f64, le_u16, le_u32, u8},
    sequence::{preceded, tuple},
    IResult,
};
use time::OffsetDateTime;

use crate::common::{boolean, osu_string, windows_datetime, OsuStr};

// TODO: A couple of fields could be represented with more meaningful structs/enums

/// Represents the `osu.db` file.
#[derive(Clone, Debug)]
pub struct BeatmapListing<'a> {
    /// osu! version (e.g. 20150203)
    pub version: u32,

    /// Folder count
    pub folder_count: u32,

    /// AccountUnlocked (only false when the account is locked or banned in any way)
    pub account_unlocked: bool,

    /// Date the account will be unlocked
    pub account_unlock_date: OffsetDateTime,

    /// Player name
    pub player_name: OsuStr<'a>,

    /// Beatmaps
    pub beatmaps: Vec<BeatmapEntry<'a>>,

    /// User permissions
    pub user_permissions: u32,
}

/// Represents a beatmap entry found in `osu.db`.
#[derive(Clone, Debug)]
pub struct BeatmapEntry<'a> {
    /// Size in bytes of the beatmap entry. Only present if version is less than 20191106.
    pub size: Option<u32>,

    /// Artist name
    pub artist_name: OsuStr<'a>,

    /// Artist name, in Unicode
    pub artist_name_unicode: OsuStr<'a>,

    /// Song title
    pub song_title: OsuStr<'a>,

    /// Song title, in Unicode
    pub song_title_unicode: OsuStr<'a>,

    /// Creator name
    pub creator_name: OsuStr<'a>,

    /// Difficulty (e.g. Hard, Insane, etc.)
    pub difficulty: OsuStr<'a>,

    /// Audio file name
    pub audio_filename: OsuStr<'a>,

    /// MD5 hash of the beatmap
    pub md5: OsuStr<'a>,

    /// Name of the .osu file corresponding to this beatmap
    pub beatmap_filename: OsuStr<'a>,

    /// Ranked status (0 = unknown, 1 = unsubmitted, 2 = pending/wip/graveyard, 3 = unused, 4 = ranked, 5 = approved, 6 = qualified, 7 = loved)
    pub ranked_status: RankedStatus,

    /// Number of hitcircles
    pub hitcircle_count: u16,

    /// Number of sliders (note: this will be present in every mode)
    pub slider_count: u16,

    /// Number of spinners (note: this will be present in every mode)
    pub spinner_count: u16,

    /// Last modification time, Windows ticks
    pub last_modification_time: OffsetDateTime,

    /// Approach rate. Byte if the version is less than 20140609, Single otherwise.
    pub approach_rate: f32,

    /// Circle size. Byte if the version is less than 20140609, Single otherwise.
    pub circle_size: f32,

    /// HP drain. Byte if the version is less than 20140609, Single otherwise.
    pub hp_drain: f32,

    /// Overall difficulty. Byte if the version is less than 20140609, Single otherwise.
    pub overall_difficulty: f32,

    /// Slider velocity
    pub slider_velocity: f64,

    /// Star Rating info for osu! standard
    pub star_ratings_std: Vec<(u32, f64)>,

    /// Star Rating info for Taiko
    pub star_ratings_taiko: Vec<(u32, f64)>,

    /// Star Rating info for CTB
    pub star_ratings_ctb: Vec<(u32, f64)>,

    /// Star Rating info for osu!mania
    pub star_ratings_mania: Vec<(u32, f64)>,

    /// Drain time, in seconds
    pub drain_time: u32,

    /// Total time, in milliseconds
    pub total_time: u32,

    /// Time when the audio preview when hovering over a beatmap in beatmap select starts, in milliseconds
    pub audio_preview_time: u32,

    /// Timing points
    pub timing_points: Vec<TimingPoint>,

    /// Difficulty ID
    pub difficulty_id: u32,

    /// Beatmap ID
    pub beatmap_id: u32,

    /// Thread ID
    pub thread_id: u32,

    /// Grade achieved in osu! standard
    pub grade_std: u8,

    /// Grade achieved in taiko
    pub grade_taiko: u8,

    /// Grade achieved in CTB
    pub grade_catch: u8,

    /// Grade achieved in osu!mania
    pub grade_mania: u8,

    /// Local beatmap offset
    pub local_offset: u16,

    /// Stack leniency
    pub stack_leniency: f32,

    /// osu! gameplay mode
    pub gameplay_mode: GameplayMode,

    /// Song source
    pub song_source: OsuStr<'a>,

    /// Song tags
    pub song_tags: OsuStr<'a>,

    /// Online offset
    pub online_offset: u16,

    /// Font used for the title of the song
    pub font: OsuStr<'a>,

    /// Is beatmap unplayed
    pub is_unplayed: bool,

    /// Last time when beatmap was played
    pub last_played: OffsetDateTime,

    /// Is the beatmap osz2
    pub is_osz2: bool,

    /// Folder name of the beatmap, relative to Songs folder
    pub folder_name: OsuStr<'a>,

    /// Last time when beatmap was checked against osu! repository
    pub last_checked_online: OffsetDateTime,

    /// Ignore beatmap sound
    pub ignore_beatmap_hitsounds: bool,

    /// Ignore beatmap skin
    pub ignore_beatmap_skin: bool,

    /// Disable storyboard
    pub disable_storyboard: bool,

    /// Disable video
    pub disable_video: bool,

    /// Mania scroll speed
    pub mania_scroll_speed: u8,
}

/// Represents the ranked status of a beatmap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankedStatus {
    Unknown = 0,
    Unsubmitted = 1,

    /// Pending / WIP / Graveyard
    Pending = 2,

    // NOTE: 3 is unused
    Ranked = 4,
    Approved = 5,
    Qualified = 6,
    Loved = 7,
}

/// Represents the different gameplay modes for a beatmap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayMode {
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

/// Represents a timing point found in `osu.db`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimingPoint {
    /// The BPM of this timing point.
    pub bpm: f64,

    /// The offset into the song.
    pub song_offset: f64,

    /// Whether this timing point is inherited.
    pub inherited: bool,
}

/// Parses an `osu.db` file.
fn beatmap_listing<'a>(input: &'a [u8]) -> IResult<&'a [u8], BeatmapListing<'a>> {
    let (rest, version) = le_u32(input)?;
    let (rest, folder_count) = le_u32(rest)?;
    let (rest, account_unlocked) = boolean(rest)?;
    let (rest, account_unlock_date) = windows_datetime(rest)?;
    let (rest, player_name) = osu_string(rest)?;

    let (rest, beatmap_count) = le_u32(rest)?;
    let (rest, beatmaps) = count(beatmap_entry(version), beatmap_count as usize)(rest)?;

    let (rest, user_permissions) = le_u32(rest)?;

    Ok((
        rest,
        BeatmapListing {
            version,
            folder_count,
            account_unlocked,
            account_unlock_date,
            player_name,
            beatmaps,
            user_permissions,
        },
    ))
}

fn beatmap_entry<'a>(version: u32) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], BeatmapEntry<'a>> {
    let parse_difficulty: fn(&[u8]) -> IResult<&[u8], f32> = if version < 20140609 {
        |i: &[u8]| map(u8, |b| b as f32)(i)
    } else {
        |i: &[u8]| le_f32(i)
    };

    move |input| {
        let (rest, size) = if version < 20191106 {
            map(le_u32, |s| Some(s))(input)?
        } else {
            (input, None)
        };

        let (rest, artist_name) = osu_string(rest)?;
        let (rest, artist_name_unicode) = osu_string(rest)?;
        let (rest, song_title) = osu_string(rest)?;
        let (rest, song_title_unicode) = osu_string(rest)?;
        let (rest, creator_name) = osu_string(rest)?;
        let (rest, difficulty) = osu_string(rest)?;
        let (rest, audio_filename) = osu_string(rest)?;
        let (rest, md5) = osu_string(rest)?;
        let (rest, beatmap_filename) = osu_string(rest)?;
        let (rest, ranked_status) = ranked_status(rest)?;

        let (rest, hitcircle_count) = le_u16(rest)?;
        let (rest, slider_count) = le_u16(rest)?;
        let (rest, spinner_count) = le_u16(rest)?;
        let (rest, last_modification_time) = windows_datetime(rest)?;
        let (rest, approach_rate) = parse_difficulty(rest)?;
        let (rest, circle_size) = parse_difficulty(rest)?;
        let (rest, hp_drain) = parse_difficulty(rest)?;
        let (rest, overall_difficulty) = parse_difficulty(rest)?;
        let (rest, slider_velocity) = le_f64(rest)?;
        let (rest, star_ratings_std) = star_ratings(rest)?;

        let (rest, star_ratings_taiko) = star_ratings(rest)?;
        let (rest, star_ratings_ctb) = star_ratings(rest)?;
        let (rest, star_ratings_mania) = star_ratings(rest)?;
        let (rest, drain_time) = le_u32(rest)?;
        let (rest, total_time) = le_u32(rest)?;
        let (rest, audio_preview_time) = le_u32(rest)?;

        let (rest, timing_point_count) = le_u32(rest)?;
        let (rest, timing_points) = count(timing_point, timing_point_count as usize)(rest)?;

        let (rest, difficulty_id) = le_u32(rest)?;
        let (rest, beatmap_id) = le_u32(rest)?;
        let (rest, thread_id) = le_u32(rest)?;
        let (rest, grade_std) = u8(rest)?;
        let (rest, grade_taiko) = u8(rest)?;
        let (rest, grade_catch) = u8(rest)?;
        let (rest, grade_mania) = u8(rest)?;
        let (rest, local_offset) = le_u16(rest)?;
        let (rest, stack_leniency) = le_f32(rest)?;
        let (rest, gameplay_mode) = gameplay_mode(rest)?;

        let (rest, song_source) = osu_string(rest)?;
        let (rest, song_tags) = osu_string(rest)?;
        let (rest, online_offset) = le_u16(rest)?;
        let (rest, font) = osu_string(rest)?;
        let (rest, is_unplayed) = boolean(rest)?;
        let (rest, last_played) = windows_datetime(rest)?;
        let (rest, is_osz2) = boolean(rest)?;
        let (rest, folder_name) = osu_string(rest)?;
        let (rest, last_checked_online) = windows_datetime(rest)?;
        let (rest, ignore_beatmap_hitsounds) = boolean(rest)?;

        let (rest, ignore_beatmap_skin) = boolean(rest)?;
        let (rest, disable_storyboard) = boolean(rest)?;
        let (rest, disable_video) = boolean(rest)?;

        // NOTE: Unused f32 optional field, only present if version is less than 20140609
        let (rest, _) = cond(version < 20140609, le_f32)(rest)?;

        // NOTE: Unused u32 field (appears to be last modification time as well)
        let (rest, _) = le_u32(rest)?;

        let (rest, mania_scroll_speed) = u8(rest)?;

        Ok((
            rest,
            BeatmapEntry {
                size,
                artist_name,
                artist_name_unicode,
                song_title,
                song_title_unicode,
                creator_name,
                difficulty,
                audio_filename,
                md5,
                beatmap_filename,
                ranked_status,
                hitcircle_count,
                slider_count,
                spinner_count,
                last_modification_time,
                approach_rate,
                circle_size,
                hp_drain,
                overall_difficulty,
                slider_velocity,
                star_ratings_std,
                star_ratings_taiko,
                star_ratings_ctb,
                star_ratings_mania,
                drain_time,
                total_time,
                audio_preview_time,
                timing_points,
                difficulty_id,
                beatmap_id,
                thread_id,
                grade_std,
                grade_taiko,
                grade_catch,
                grade_mania,
                local_offset,
                stack_leniency,
                gameplay_mode,
                song_source,
                song_tags,
                online_offset,
                font,
                is_unplayed,
                last_played,
                is_osz2,
                folder_name,
                last_checked_online,
                ignore_beatmap_hitsounds,
                ignore_beatmap_skin,
                disable_storyboard,
                disable_video,
                mania_scroll_speed,
            },
        ))
    }
}

/// Parses a ranked status value.
fn ranked_status(input: &[u8]) -> IResult<&[u8], RankedStatus> {
    use RankedStatus::*;

    let (rest, status) = u8(input)?;
    let status = match status {
        0 => Unknown,
        1 => Unsubmitted,
        2 => Pending,
        4 => Ranked,
        5 => Approved,
        6 => Qualified,
        7 => Loved,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Switch,
            }))
        }
    };

    Ok((rest, status))
}

/// Parses a gameplay mode value.
fn gameplay_mode(input: &[u8]) -> IResult<&[u8], GameplayMode> {
    use GameplayMode::*;

    let (rest, status) = u8(input)?;
    let status = match status {
        0 => Standard,
        1 => Taiko,
        2 => Catch,
        3 => Mania,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Switch,
            }))
        }
    };

    Ok((rest, status))
}

/// Parses a integer-double pair found in `osu.db`.
fn int_double_pair(input: &[u8]) -> IResult<&[u8], (u32, f64)> {
    let (rest, int) = preceded(tag(&[0x08]), le_u32)(input)?;
    let (rest, double) = preceded(tag(&[0x0d]), le_f64)(rest)?;

    Ok((rest, (int, double)))
}

/// Parses a timing point found in `osu.db`.
fn timing_point(input: &[u8]) -> IResult<&[u8], TimingPoint> {
    map(
        tuple((le_f64, le_f64, boolean)),
        |(bpm, song_offset, inherited)| TimingPoint {
            bpm,
            song_offset,
            inherited,
        },
    )(input)
}

/// Parses a list of star ratings.
fn star_ratings(input: &[u8]) -> IResult<&[u8], Vec<(u32, f64)>> {
    let (rest, total) = le_u32(input)?;
    count(int_double_pair, total as usize)(rest)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ranked_status_decoding_works() {
        use RankedStatus::*;

        assert_eq!(ranked_status(&[0]), Ok((&[][..], Unknown)));
        assert_eq!(ranked_status(&[1]), Ok((&[][..], Unsubmitted)));
        assert_eq!(ranked_status(&[2]), Ok((&[][..], Pending)));
        assert_eq!(ranked_status(&[4]), Ok((&[][..], Ranked)));
        assert_eq!(ranked_status(&[5]), Ok((&[][..], Approved)));
        assert_eq!(ranked_status(&[6]), Ok((&[][..], Qualified)));
        assert_eq!(ranked_status(&[7]), Ok((&[][..], Loved)));

        assert_eq!(
            ranked_status(&[10]),
            Err(nom::Err::Error(nom::error::Error {
                input: &[10][..],
                code: nom::error::ErrorKind::Switch
            }))
        );
    }

    #[test]
    fn gameplay_mode_decoding_works() {
        use GameplayMode::*;

        assert_eq!(gameplay_mode(&[0]), Ok((&[][..], Standard)));
        assert_eq!(gameplay_mode(&[1]), Ok((&[][..], Taiko)));
        assert_eq!(gameplay_mode(&[2]), Ok((&[][..], Catch)));
        assert_eq!(gameplay_mode(&[3]), Ok((&[][..], Mania)));

        assert_eq!(
            gameplay_mode(&[10]),
            Err(nom::Err::Error(nom::error::Error {
                input: &[10][..],
                code: nom::error::ErrorKind::Switch
            }))
        );
    }

    #[test]
    fn int_double_pair_decoding_works() {
        let int: u32 = 100;
        let double: f64 = 1234.56;
        let extra = [0x01, 0x02, 0x03];

        let mut pair = Vec::new();
        pair.push(0x08);
        pair.extend_from_slice(&int.to_le_bytes());
        pair.push(0x0d);
        pair.extend_from_slice(&double.to_le_bytes());
        pair.extend_from_slice(&extra);

        let mut missing_front_tag = Vec::new();
        missing_front_tag.extend_from_slice(&int.to_le_bytes());
        missing_front_tag.extend_from_slice(&double.to_le_bytes());

        let mut missing_middle_tag = Vec::new();
        missing_middle_tag.push(0x08);
        missing_middle_tag.extend_from_slice(&int.to_le_bytes());
        missing_middle_tag.extend_from_slice(&double.to_le_bytes());

        assert_eq!(int_double_pair(&pair), Ok((&extra[..], ((int, double)))));

        assert_eq!(
            int_double_pair(&missing_front_tag),
            Err(nom::Err::Error(nom::error::Error {
                input: &missing_front_tag[..],
                code: nom::error::ErrorKind::Tag
            }))
        );

        assert_eq!(
            int_double_pair(&missing_middle_tag),
            Err(nom::Err::Error(nom::error::Error {
                input: &double.to_le_bytes()[..],
                code: nom::error::ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn timing_point_decoding_works() {
        let bpm: f64 = 180.0;
        let song_offset: f64 = 250.0;
        let inherited = true;

        let mut input = Vec::new();
        input.extend_from_slice(&bpm.to_le_bytes());
        input.extend_from_slice(&song_offset.to_le_bytes());
        input.push(0x01);

        // Extra data
        input.extend_from_slice(&[0x05, 0x06]);

        assert_eq!(
            timing_point(&input),
            Ok((
                &[0x05, 0x06][..],
                TimingPoint {
                    bpm,
                    song_offset,
                    inherited
                }
            ))
        );
    }

    #[test]
    fn star_ratings_decoding_works() {
        let ratings: Vec<(u32, f64)> = vec![(0, 1.2), (1, 2.3)];
        let length = ratings.len() as u32;

        let mut input = length.to_le_bytes().to_vec();

        for (mods, rating) in ratings.iter() {
            input.push(0x08);
            input.extend_from_slice(&mods.to_le_bytes());
            input.push(0x0d);
            input.extend_from_slice(&rating.to_le_bytes());
        }

        assert_eq!(star_ratings(&input), Ok((&[][..], ratings)));
    }
}