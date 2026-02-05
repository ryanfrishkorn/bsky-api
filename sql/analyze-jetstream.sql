.timer on

set timezone = 'UTC';

-- analysis of critical fields
with
totals as (
    -- total rows
    select count(*) as total_rows,

        -- createdAt extraction
        count(json_extract_string(feedpost, '$.createdAt')) as created_at_success,
        sum(case when json_extract_string(feedpost, '$.createdAt') is null then 1 else 0 end) as created_at_fail,

        -- text extraction
        count(json_extract_string(feedpost, '$.text')) as text_success,
        sum(case when json_extract_string(feedpost, '$.text') is null then 1 else 0 end) as text_fail

    from posts
),
extracted as (
    select
        json_extract_string(feedpost, '$.createdAt') as created_at,
        json_extract_string(feedpost, '$.text') as text,
        did,
        cid,
        feedpost
    from posts
),
parsed as (
    select created_at::TIMESTAMP as created_at,
        cid,
        did,
        text
    from extracted
),
parsed_counts as (
    select sum(case when try_cast(created_at as TIMESTAMP) is not null then 1 else 0 end) as created_at_success,
    sum(case when try_cast(created_at as TIMESTAMP) is null then 1 else 0 end) as created_at_fail
    from extracted
),
empty_text as (
    select count(*) as count
    from extracted
    where text = ''
),
stats as (
    select 'total_rows' as key, total_rows as value,
    from totals

    union all
    select 'created_at_json_success', created_at_success
    from totals

    union all
    select 'created_at_json_fail', created_at_fail
    from totals
    
    union all
    select 'text_success', text_success
    from totals

    union all
    select 'text_fail', text_fail
    from totals

    union all
    select 'text empty', count
    from empty_text

    union all
    select 'created_at_timestamp_success', created_at_success
    from parsed_counts

    union all
    select 'created_at_timestamp_fail', created_at_fail
    from parsed_counts

    union all
    select 'created_at_min', min(created_at)
    from extracted

    union all
    select 'created_at_max', max(created_at)
    from extracted

    union all
    select 'created_at_ts_min', min(created_at)
    from parsed

    union all
    select 'created_at_ts_max', max(created_at)
    from parsed
)
select * from stats;

-- determine the number of posts that fall outside the expected time bounds
with parsed as (
    select json_extract_string(feedpost, '$.createdAt')::TIMESTAMP as created_at,
    from posts
)
select min(created_at),
    max(created_at),
    sum(case when created_at::TIMESTAMP < strftime(now(), '%x')::TIMESTAMP then 1 else 0 end) as past, 
    sum(case when created_at::TIMESTAMP > strftime(now() + interval '1 day', '%x')::TIMESTAMP then 1 else 0 end) as future, 
    count(*) as total,
    past + future as excluded,
    strftime(now(), '%x')
from parsed;
