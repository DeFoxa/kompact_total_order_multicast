Implementing total order multicast  written using kompact actor-component hybrid concurrency model

distributed systems lecture - including FIFO, total order multicast, ISIS: https://courses.grainger.illinois.edu/ece428/sp2021//assets/slides/lect8-after.pdf


General Framework: 

    - two Kompact component types: Master and Worker. one master and multiple workers 
    - Master generates a port for request response from workers. 
        - request enum contains two types: one for RFP (request for proposal) and one for accepted sequence_number & message.
        - Master logical clock determines message delivery ordering
    - writing logic for local message handling on actors (fn receive_local). However, can be easily implemented for receive_network
    if working in distributed system of master and workers. 
