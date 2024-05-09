Total order multicast  with Kompact actor-component hybrid concurrency model

## General Framework: 
- Two Kompact component types: Master and Worker. one master and multiple workers 
- Master generates a port for request response from workers. 
    - request enum contains two types: one for RFP (request for proposal) and one for accepted sequence_number & message.
    - Master logical clock determines message delivery ordering
- writing logic for local message handling from kompact actors (fn receive_local). However, can be easily implemented for receive_network

