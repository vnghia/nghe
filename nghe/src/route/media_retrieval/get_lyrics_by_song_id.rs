use std::borrow::Cow;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::{EitherOrBoth, Itertools};
use nghe_api::media_retrieval::get_lyrics_by_song_id::{Line, Lyric, LyricsList};
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
                .map(|lyric| -> Result<_, Error> {
                    let display_title = lyric.description.map(Cow::into_owned);
                    let lang = lyric.language.into_owned();

                    if let Some(durations) = lyric.durations {
                        Ok(Lyric {
                            display_title,
                            lang,
                            synced: true,
                            offset: 0,
                            line: durations
                                .into_iter()
                                .zip_longest(lyric.texts.into_iter())
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
                        Ok(Lyric {
                            display_title,
                            lang,
                            synced: false,
                            offset: 0,
                            line: lyric
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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;
        let (id, audio) = music_folder.add_audio().call().await.database.get_index(0).unwrap();

        let n_lyrics =
            usize::from(audio.external_lyric.is_some()) + audio.information.metadata.lyrics.len();

        assert_eq!(
            handler(mock.database(), Request { id: *id })
                .await
                .unwrap()
                .lyrics_list
                .structured_lyrics
                .len(),
            n_lyrics
        );
    }
}
