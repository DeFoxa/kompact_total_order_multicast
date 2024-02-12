Implementing total order multicast using ISIS algorithm written using kompact actor-component hybrid concurrency model

Resources:
ISIS algo Theory: https://www.cs.purdue.edu/homes/bb/cs542-15Spr/Birman-Reliable-Broadcast.pdf

distributed systems lecture - including total order multicast and ISIS: https://courses.grainger.illinois.edu/ece428/sp2021//assets/slides/lect8-after.pdf


Notes: 
    - Using TUI instead of GUI for worker state visualization.

General Framework: 
    - two Kompact component types: Master and Worker. one master and multiple workers (can be any number >= 2)
    - Master generates a port for request response from workers. 
        - request enum contains two types: one for RFP (request for proposal) and one for accepted sequence_number & message.
    - writing logic for local message handling on actors (fn receive_local). However, can be easily implemented for receive_network
    if working in distributed system of master and workers. Utilizing kompact ports and Ask(...) methods, network latency/jitter 
    between master and multiple distributed workers is a relatively insignificant challenge. This is a primary benefit of Kompact.

