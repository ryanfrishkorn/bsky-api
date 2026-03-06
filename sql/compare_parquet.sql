-- Compare Parquet files to compare the number of posts captured
-- between the go jetstream client and this one.

-- create basic raw parquet views
create view posts_js1 as
    select * from 'data/archive/jetstream-2026-03-03.parquet';

create view posts_js2 as
    select * from 'data/archive/jetstream-2026-03-04.parquet';

-- parse datetime field for logical comparisons
create view js1_parsed as
    select json_extract_string(feedpost, '$.createdAt')::TIMESTAMPTZ created_at,
        *
    from posts_js1;

create view js2_parsed as
    select json_extract_string(feedpost, '$.createdAt')::TIMESTAMPTZ created_at,
        *
    from posts_js2;

-- obtain counts
select 'jetstream_1' as source, count(*) as posts from js1_parsed

union all
select 'jetstream_2', count(*) as posts from js2_parsed;
