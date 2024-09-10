# Updates

## Mar 8th

### updates
- datetime issue
    - even with basic mysql::from_value() you can't convert from mysql::Value::Date() to String
        - FromValue trait is implemented by using an intermediate type (which for String is just String)
        - so (i think) if String doesn't implement From<V> then FromValue isn't going to work for value V
            - path: /Users/alexanderportland/.cargo/registry/src/index.crates.io-6f17d22bba15001f/mysql_common-0.30.6/src/value/convert/mod.rs
    - but, for some reason, same schema is producing Date() in one case and Bytes() in another
    - IT'S got to do with prep_exec vs query

### todo for this time
- [X] get more info on timestamp issue
    - does Date() work without alohomora?
        - what works without alohomora?
        - what works with it?
        - where's the implicit conversion?
    - [X] switch pre-alohomora to use exec()
    - [X] add pre-alohomora testing
- [ ] look at potential new apps
    - [ ] [Portfolio](https://github.com/admisio/Portfolio)
    - [ ] [my-iot-rs](https://github.com/eigenein/my-iot-rs)
    - [ ] [Raider](https://github.com/valeriansaliou/raider)

### takeaways from last time
- try to solve the timestamp issue, this seems important
- doesn't have to be the biggest app, just need good policies
    - test out the testing api by using the fake test policies
    - and then only turn on the thing later

<!-- ###################################################################################### -->

## Mar 1st
### updates
- switched over everything to the new alohomora api (including testing)
    - love the new api
    - [?] how to determine which kind of region to use?? like malte said last time
        - eg sandbox, pure, pcr, etc
    - app all done now
- have a diff script for before/after comparison
- did some db work
    - fixed alohomora::db::from_value not working on dates
    - added exec (no _iter) family functions to return vectors of objects
        - and FromBBoxRow trait to do this
        - doesn't violate privacy cause the implementations of the trait will only deal with developer-facing interfaces (the BBoxRow type)
        * worth the time to look into deriving this more? or probably okay that developers would have to write this themselves
- what to do next??
    - rustodon -> social media
        - mature, doesn't seem to be getting updates
        - isn't set up to run with apple silicon
        - but has very reasonable LOC
    - Plume -> 
        - probably too large? (~x20 of youchat)


### todo for this time
- [X] port over app to new alohomora api
    - [X] merge in prep_exec_drop function (forgot to push before kinan did last time)
- [X] switch to new alohomora testing api
- [X] clean up code
- [X] take a look at differences between normal and alohomora
- [X] db changes
    - [X] prep_exec returns BBoxQueryResult object rather than Vec<T>
        - jank to access first element (bc no .first())
        - causing unneccessary loops (eg groupchat.rs line 76-85)
    - [=>] need some way to convert tuples of bboxes into structs
- [X] rexamine db timestamp issue
- [ ] take last pass to clean up, shorten new alohomora db code
- [ ] start looking for another application to port
    - has to be rocket & rust => get justus' list from kinan

<!-- ###################################################################################### -->

## Feb 23rd
### updates
- got the whole thing working in alohomora w/ policies
    - something weird with timestamps being converted to dates rather than bytes
    - no .first() in our version of query result
- got testing going
    - everything but buggy tests pass without alohomora, everything passes with
    - any other edge cases to test?

<!-- ###################################################################################### -->