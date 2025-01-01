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
        .select(lyrics::Data::as_select())
        .get_results(&mut database.get().await?)
        .await?;
    Ok(Response {
        lyrics_list: LyricsList {
            structured_lyrics: lyrics
                .into_iter()
                .map(|lyrics| -> Result<_, Error> {
                    if let Some(durations) = lyrics.durations {
                        Ok(Lyrics {
                            lang: lyrics.language.into_owned(),
                            synced: true,
                            line: durations
                                .into_iter()
                                .zip_longest(lyrics.texts.into_iter())
                                .map(|iter| {
                                    if let EitherOrBoth::Both(duration, text) = iter {
                                        Ok(Line {
                                            start: Some(duration.try_into()?),
                                            value: text.into_owned(),
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
                                .texts
                                .into_iter()
                                .map(|text| Line { start: None, value: text.into_owned() })
                                .collect(),
                        })
                    }
                })
                .try_collect()?,
        },
    })
}
