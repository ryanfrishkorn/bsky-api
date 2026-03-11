-- Compare Parquet files to compare the number of posts captured
-- between the go jetstream client and this one.

-- create basic raw parquet views
create view posts_js1 as
    select * from 'data/archive/jetstream-2026-03-10_mini.parquet';

create view posts_js2 as
    select * from 'data/archive/jetstream-2026-03-10.parquet';

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

-- compare by hour
create temporary table js1 as
    select substr(created_at::VARCHAR, 0, 14) as ts, count(*) as count
    from js1_parsed group by ts
    order by ts;

create temporary table js2 as
    select substr(created_at::VARCHAR, 0, 14) as ts, count(*) as count
    from js2_parsed group by ts
    order by ts;

create temporary table comparison as
    select js1.ts, js1.count as file1, js2.count as file2, abs(js1.count - js2.count) as disparity from js1 left join js2 on js1.ts = js2.ts;

select * from comparison;

select 'cumulative disparity' as key, abs(sum(disparity)) as value from comparison;
