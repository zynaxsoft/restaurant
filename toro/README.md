# Tanapol's Obvious Restaurant Order (TORO)
- Format
    - `<command> for table <table-id>[: <arg1>, <arg2>]`
    - Everything is case sensitive
    - Exactly one white space where it is needed, like what a normal people would do.
- Add
    - `new order for table <table-id>: <menu> * <quantity>, <menu> * <quantity>, ...`
- Remove
    - `cancel for table <table-id>: <menu> * <quantity>, <menu> *  <quantity>, ...`
    - `yeet`
        - cancel everything in the restaurant (used in demo)
- Query
    - `check for table <table-id>`
    - `check for table <table-id>: <menu>, <menu>, ...`