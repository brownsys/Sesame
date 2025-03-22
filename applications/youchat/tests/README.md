### testing strategy
broad strokes:
- ``basic_tests.rs`` -> users can send and recieve basic chats
    - send message to someone, they can see it
    - send message to someone, someone else can't see it
- ``groupchat_tests.rs`` -> users can send and recieve group chats
    - send message to group, members can see it
    - send message to group, non-members can't
- ``buggy_tests.rs`` -> the buggy endpoint fails
    - buggy endpoint always fails
    - unless all chats are yours anyway
        - but even then, it will fail if anyone else sends a message