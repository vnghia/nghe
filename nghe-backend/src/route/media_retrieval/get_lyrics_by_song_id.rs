use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::{EitherOrBoth, Itertools};
use nghe_api::media_retrieval::get_lyrics_by_song_id::{Line, Lyrics, LyricsList};
pub use nghe_api::media_retrieval::get_lyrics_by_song_id::{Request, Response};
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::orm::{lyrics, songs};
use crate::{Error, error};

#[handler]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let lyrics = lyrics::table
        .inner_join(songs::table)
        .filter(songs::id.eq(request.id))
        .select(lyrics::Lyrics::as_select())
        .get_results(&mut database.get().await?)
        .await?;
    Ok(Response {
        lyrics_list: LyricsList {
            structured_lyrics: lyrics
                .into_iter()
                .map(|lyrics| -> Result<_, Error> {
                    if let Some(line_starts) = lyrics.line_starts {
                        Ok(Lyrics {
                            lang: lyrics.language.into_owned(),
                            synced: true,
                            line: line_starts
                                .into_iter()
                                .zip_longest(lyrics.line_values.into_iter())
                                .map(|iter| {
                                    if let EitherOrBoth::Both(start, value) = iter {
                                        Ok(Line {
                                            start: Some(start.try_into()?),
                                            value: value.into_owned(),
                                        })
                                    } else {
                                        error::Kind::DatabaseCorruptionDetected.into()
                                    }
                                })
                                .try_collect()?,
                        })
                    } else {
                        Ok(Lyrics {
                            lang: lyrics.language.into_owned(),
                            synced: false,
                            line: lyrics
                                .line_values
                                .into_iter()
                                .map(|value| Line { start: None, value: value.into_owned() })
                                .collect(),
                        })
                    }
                })
                .try_collect()?,
        },
    })
}
