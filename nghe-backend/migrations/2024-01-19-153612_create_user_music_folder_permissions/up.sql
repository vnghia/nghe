-- Your SQL goes here
create table
user_music_folder_permissions (
    user_id uuid not null,
    music_folder_id uuid not null,
    allow boolean not null default true,
    constraint user_music_folder_permissions_pkey primary key (
        user_id, music_folder_id
    ),
    constraint user_music_folder_permissions_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade,
    constraint user_music_folder_permissions_music_folder_id_fkey foreign key (
        music_folder_id
    ) references music_folders (id) on delete cascade
);

create index user_music_folder_permissions_user_id_idx on user_music_folder_permissions (
    user_id
);

create index user_music_folder_permissions_music_folder_id_idx on user_music_folder_permissions (
    music_folder_id
);
