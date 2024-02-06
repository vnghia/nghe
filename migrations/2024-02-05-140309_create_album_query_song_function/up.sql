-- Your SQL goes here
create function album_query_song_by_music_folder (_music_folder_ids uuid[], _album_id uuid) returns table (song_id uuid) as $$
  select
    s.id
  from
    songs as s
  where
    s.album_id = _album_id
    and s.music_folder_id = any (_music_folder_ids)
$$ language sql stable leakproof parallel safe;

create function album_query_song_by_user (_user_id uuid, _album_id uuid) returns table (song_id uuid) as $$
  select album_query_song_by_music_folder (
    array (
      select
        umfp.music_folder_id
      from
        user_music_folder_permissions as umfp
      where
        umfp.user_id = _user_id
        and umfp.allow is true
      ),
    _album_id
  )
$$ language sql stable leakproof parallel safe;

create function album_count_song_by_music_folder (_music_folder_ids uuid[], _album_id uuid) returns setof bigint as $$
  select count(*) from album_query_song_by_music_folder (_music_folder_ids, _album_id)    
$$ language sql stable leakproof parallel safe;

create function album_count_song_by_user (_user_id uuid, _album_id uuid) returns setof bigint as $$
  select count(*) from album_query_song_by_user (_user_id, _album_id)
$$ language sql stable leakproof parallel safe;