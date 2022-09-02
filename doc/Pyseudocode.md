Pyseudocode 

For three normal phase, just follow the paper.

### Retransmission feature

Like heartbeat, broadcast periodically, it can be used letting node know they should start state transfer.

Example: 

assume: L = 6, k = 2

A: 2 3 4 5 6 7 8

B: 2 3 4 5 6 7 8

C: 2 3 4 5 6 7 8

D: 0 1 2 3 4 5 6

D lags.

**In normal status**

broadcast to all nodes: \<STATUS-ACTIVE, v, h , le, i, log_status>

le: sequence number I last applied

log_status: contains prepared or commited from (le ~ h + L)

When a replica receive it, it reset the timer(avoid ddos) for the sender.



**when receive retransmission msg STATUS-ACTIVE**:

check msg.log_status, if node.n.status > msg.n.status, add this sequence number to a set, and send this set to the sender of retrainsmission.





**In view change status TODO 下周写**

broadcast to all nodes: \<STATUS-PENDING v, h , le, i, p, c>





### Checkpoint management and state transfer

This repository's checkpoint management is not as complicated as paper.

When a replica find they receive 2f + 1 prepare, commit msg whose sequence number is higher than node's high water mark. This means I lag.

```
when receive prepare or commit, 
if map[n] == 2f + 1:
	

```



state transfer:

The replica may learn about such a checkpoint by receiving checkpoint messages or as the result of a view change.

⟨CHECKPOINT, *n*, *d* , *i*⟩α*i*
