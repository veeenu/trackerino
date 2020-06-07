## General structure of the tracker

- One endpoint which sends Javascript code over to the browser
- One endpoint which receives tracking information via URI
- One authenticated endpoint group for extracting and arranging tracking information
  -> login/logout endpoints
  -> N endpoints for some timeseries measures:
     -> Users by geographical location (IP lookup)
     -> Users by useragent
     -> Users by operating system
     -> Later I'll inspect `document` from chrome's console to see what's in there
- Static dashboard files consuming the endpoint above

SQLite as database because this is going to be a self hosted thingy

## Persistence

### Design for quick and dirty

- Single SQLite file, concurrent access, no backpressure control

### Design for high performance

- Buffer requests in  a thread, manage pressure by writing to  a number of other
  databases, then  sync them up  (we should never come  to that unless  this thing
  powers Amazon)

## Misc

I want to see whether there's a non-intrusive, non-third-party-service-dependent
way of telling a region from an IP. Maybe I could to it later, doesn't seem that
big of a  priority. I also need to  figure out if all of this  is GDPR compliant
and how  much data I  can glean from a  request without it  constituting private
information tracking
