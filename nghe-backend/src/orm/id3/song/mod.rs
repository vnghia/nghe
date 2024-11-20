pub mod id_duration;

use diesel::prelude::*;
pub use id_duration::IdDuration;
use nghe_api::common::format::Trait;
use nghe_api::id3;
use nghe_api::id3::builder::song as builder;
use num_traits::ToPrimitive;
use uuid::Uuid;

use crate::orm::songs;
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    #[diesel(embed)]
    pub track: songs::position::Track,
    #[diesel(embed)]
    pub date: songs::date::Date,
    #[diesel(embed)]
    pub file: songs::property::File,
    #[diesel(embed)]
    pub property: songs::property::Property,
    #[diesel(embed)]
    pub disc: songs::position::Disc,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
}

pub type BuilderSet = builder::SetMusicBrainzId<
    builder::SetDiscNumber<
        builder::SetChannelCount<
            builder::SetSamplingRate<
                builder::SetBitDepth<
                    builder::SetBitRate<
                        builder::SetDuration<
                            builder::SetSuffix<
                                builder::SetContentType<
                                    builder::SetSize<
                                        builder::SetYear<
                                            builder::SetTrack<builder::SetTitle<builder::SetId>>,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;

impl Song {
    pub fn try_into_api_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(id3::song::Song::builder()
            .id(self.id)
            .title(self.title)
            .maybe_track(self.track.number.map(u16::try_from).transpose()?)
            .maybe_year(self.date.year.map(u16::try_from).transpose()?)
            .size(self.file.size.cast_unsigned())
            .content_type(self.file.format.mime())
            .suffix(self.file.format.extension())
            .duration(
                self.property
                    .duration
                    .ceil()
                    .to_u32()
                    .ok_or_else(|| Error::CouldNotConvertFloatToInteger(self.property.duration))?,
            )
            .bit_rate(self.property.bitrate.try_into()?)
            .maybe_bit_depth(self.property.bit_depth.map(u8::try_from).transpose()?)
            .sampling_rate(self.property.sample_rate.try_into()?)
            .channel_count(self.property.channel_count.try_into()?)
            .maybe_disc_number(self.disc.number.map(u16::try_from).transpose()?)
            .maybe_music_brainz_id(self.music_brainz_id))
    }

    pub fn try_into_api(self) -> Result<id3::song::Song, Error> {
        Ok(self.try_into_api_builder()?.build())
    }
}
