delete from playlists_songs
where playlists_songs.playlist_id = $1 and playlists_songs.song_id = any(
    select song_id
    from
        (
            select
                ps.song_id,
                row_number() over (
                    partition by ps.playlist_id
                    order by
                        ps.created_at asc
                ) as pos
            from
            (
                playlists_songs as ps
                inner join songs as s on (ps.song_id = s.id)
            )
            where
                (
                    exists (
                        select
                            umfp.user_id,
                            umfp.music_folder_id
                        from
                            user_music_folder_permissions as umfp
                        where
                            (
                                (umfp.user_id = $2)
                                and (
                                    umfp.music_folder_id = s.music_folder_id
                                )
                            )
                    )
                    and (
                        ps.playlist_id = $1
                    )
                )
        )
    where
        pos = any($3)
);
