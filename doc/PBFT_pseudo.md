### Pseudo-code of pbft for mac implementation



**Some Macros:**





#### log data structure

```
log_entry:
	int type																// PRE-PREPARE, PREPARE, COMMIT, APPLIED, etc...
	int v																		// view number
	int n																		// sequence number
	int client															// client‘s number
	int who_send														// who send this msg
	int cert_prepare_num										// prepare certificate number
	bool[clients_num]	cert_prepare_vote			// who send prepare certificate
	int cert_commit_num											// commit certificate number
	bool[clients_num]	cert_commit_vote			// who send commit certificate
```

```
view_change_log_entry:
	int v
	int h
	list<n, d> C
	list<n,d,v> P
	list<n,d,v> Q
	int i
	list<bool>	acks
	ack_nums
```



The vote and vote_digest try to solve two problems:

(1) If a replica receives leader's pre-prepare msg, however, it doesn't recived client request, in this case, the replica will reject it, even though it is good node. So the failure model broken.

(2) If client is malicious and it never send his request to leader, view change would happens forever.

**Solution:**

**Backup:**

After a replica receives a client, it stores request in corresponding data structure

send **Accept_Client<c, i, digest>** to leader,



**Leader:**

Receive it and vote[i] = 1, since one replica only can vote onece for one specific c.request

Then map[c].vote_digest[digest]+=1, if the value == 2f + 1, means a quorum remember it. Then it is good to process normal phase.

 **leader:**

```
client_request_entry:
	request,
	digest,
	timestamp,
	status,
	list<bool> vote,
	map<digestm:num> vote_digest	
```



#### Replicas data structure

```
node:
	// normal variables
	I_am																									//number of replicas
	
	map<client: client_request_entry> client_request 	
  int my_view
 	list<log_entry> log			 															// record logs related 	information
 	int applied																						// apply pointer
 	timer																									// record request process dudation
 	
  // backup variables
  int who_leader																				// who is leader
  
  // leader variables
  int log_assign																			// point to the empty usable log slot
  
  
  // garbage collcetion and checkpoint
  h 																						//sequence number for latest checkpoint + 1
  L																							// slid window size
  
  // view change part
  list<view_change_log>  view_change_log size: num(replicas)
  view_change_num
  list<n,d> prepare size: L
```

we don't need to consider outbound of set<c, latest_request_timestemp>, since client number is limited in PBFT.



### Request send:

Client: client sends request to every replica, with timestamp which guarantees one execution semantics. Message type: ⟨REQUEST, o, t, c⟩αc   o: operation, t: timestamp, c: client 

TODO: one semantic, cluster may lose the client msg, for example, VIEW-CHANGE before leader send pp.

It needs more consideration.

```
when sends the message to cluster:
  broadcast ⟨REQUEST, o, t, c⟩αc to all replicas, blocked or (timeout and resend it with same t).
  wait for f + 1 commit
  if timeout: resend it 
  do next operations
```



Leader and backups:

```
Received message: 
  verify  ⟨REQUEST, o, t, c⟩αc signature by c's public key, if failed, ommit it and tell client the status.
  
  // check timestamp
  //maintain one semantic: TODO need more consideration
  if c in node.client_request:
  	if node.client_request[c].timestamp == request's timestamp
    	and node.client_request[c].status != canceled : 
  		ommit it, return message like"we are processing your request".
  else:
  	//store it
  	calculate digest of msg
  	client_request_entry = node.client_request[c]
		client_request_entry.request = request.request
		client_request_entry.digest = request.digest,
		client_request_entry.timpstamp = request.timsamp,
		client_request_entry.status = INIT,
		client_request_entry.vote[I_am] = 1,
		vote_digest[digest] += 1
  	
  if I am backup:
  	send Accept_Client<c, i, digest> to leader
    if timeout: make VIEW-CHANGE
```

only for Leader

```
Receive Accept_Client<c, i, digest>:
  	client_request_entry = node.client_request[c]
		if client_request_entry.vote[i] == 1 or client_request_entry.status != INIT: 
			ommit it
		client_request_entry.vote[i] = 1,
		vote_digest[digest] += 1
		if vote_digest[digest] == 2f + 1: client_request_entry.status = PRE-PREPARED
		GO TO PRE-PREPARE phase.

```



### checkpoint and logs(in memory) management

From papar

> The low water mark *h* is equal to the sequence number of the last stable checkpoint and the high water mark is *H* = *h* + *L*, where *L* is the log size. The log size is the maximum number of consecutive sequence numbers for which the replica will log information. It is obtained by multiplying *K* by a small constant factor (e.g., 2) that is big enough so that it is unlikely for replicas to stall waiting for a checkpoint to become stable.
>
> Generating these proofs after executing every operation would be expensive. Instead, they are generated periodically, when a request with a sequence num- ber divisible by the *checkpoint period K* is executed (e.g., *K* = 128). We refer to the states produced by the execution of these requests as *checkpoints* and we say that a checkpoint with a proof is a *stable checkpoint*.

Therefore, we only need to mantain a slid window whose size = L. 

Since pre-prepared, prepare, commited message has the similar format: <msg, v, n, d, i>, so the list to store them has format: list`<msg, v, n, d, i, status>`, I name it as `logs` ,status variables has 4 states pre-prepared, prepared, commited, applied.

Everytime a replica commits a request, it check wether applied pointer can moved, if `logs[applied + 1] .status == commited`, apply it, and apply pointer moves forward.

When` logs[apply].status == applied and n % L == 0 `, that means it is a stable checkpoint, we store it into disk, and truncate it from logs.

Example:

| <msg1, v,n(h),d,i, applied> | <msg2, v, n + 1, d, i, commied> | <msg3, v, n + 2,d, i, pre-prepared> | <msg2, v,n + 3, d, i, commited> | <msg2, v, n + 4,d,i,prepared> | <msg2, v, n + 5,d,i,commited> |
| --------------------------- | ------------------------------- | ----------------------------------- | ------------------------------- | ----------------------------- | ----------------------------- |

applypointer points to op1

Msg0(which alreayd is truncated) is stable checkpoint, we assume L = 6. Everytime one msg is commited, applied pointer tries to move forward, and execte corresponding msg. In this example, the next executed operation is msg2. Besides, after it applied, it replies to the corresponding client.



TODO: how to solve the problem that slid window stucks:

For example, f = 1, 2f + 1 = 3, 3f + 1 = 4, . So there are 1, 2, 3, 4 replicas

L = 2

In current slid window: `n, n + 1`, two request should be processed.

However, `n` is prepared by 1 2 3, `n + 1` is prepared by 2, 3 ,4. In this case, we satisfy both of two requests are memoried by quorum, and leader is good, but we we can't move the window, since two quorum need information in another quorum. So, my solution is, when timer nearly out, replicas should fetch states from other replicas. For example, 1 2 3 should fetch from `n + 1`'s prepared certificates from 2 3 4. if successful, window can slid, leader is good, we can process the new request. If not, leader may faulty, make VIEW-CHANGE.  



### PRE-PREARE phase:

Msg format: ⟨PRE-PREPARE, c, v, n, D(m)⟩αp, v: view, n:sequence number, D(m) digest

Leader:

```
//check wether h~h+L has slot for this msg format, if no room, ,omit it,retuan,(wait for CHECKPOINT-FETCH or VIEW-CHANGE)
if log_assigned > h + L: TODO: make log fetch?

// create corresponding log and store it
cur_log = {
						type = PRE-PREPARED, 
					  v = my_view,
					  n = log_assign,
					  client = REQUEST.c,
					  who_send = I_am,
					  cert_prepare_num = 1,
					  cert_prepare_vote[I_am] = True,
					  cert_commit_num = 0,
					  cert_commit_vote, // do nothing						
						}
node.logs[log_assign] = cur_log
log_assign += 1
node.client_request[c].status = PRE-PREPARED
multicasts ⟨PRE-PREPARED, c, v, n, D(m)⟩αp
```

Backup:

```
assume msg's name = pre-prepare
When receive pre-prepare msg:
	//verify sender's identification
	if mac key can't unwrap msg, return
	// check digest
	if client_request[c].digest != pre-prepare.D(m): ommit, return
	// check v
	if pre-prepare.view != my_view.v: return
	// check h + L
	if n <= h or n > h + L: return

	if logs[pre-prepare.n] is not used:
    cur_log = {
              type = PRE-PREPARED, 
              v = pre-prepare.v,
              n = pre-prepare.n,
              client = pre-prepare.c,
              who_send = who_leader,
              cert_prepare_num = 2,
              cert_prepare_vote[I_am, who_leader] = True,
              cert_commit_num = 0,
              cert_commit_vote, // do nothing						
              }
      node.client_request[c].status = PRE-PREPARED
      node.logs[pre-prepare.n] = cur_log
      multicast (PREPARE, msg, v, n, d, i)
      go to PREPARE phase
```



### PREPARE

Backup and leader:  prepare msg (msg, v, n,  d, i, PREPARE)

why we need digest in prepare msg? since leader may faulty, if msg's digest != log's digest, replica should not to apply it.

```
assume msg's name is prepare
When receive prepare msg
	//verify sender's identification
	if mac key can't unwrap msg, return
	// check v
	if pre-prepare.view != pre-prepare.v: return
	// check h + L
	if n <= h or n > h + L: return
  // check prepare msg with corresponding pre-prepared msg
  if logs[prepare.n].digest != prepare.digest: return
  
  // same question like pre-prepare
  
  //normal case
  if node.logs[pre-prepare.n].status != PRE-PREPARED: return //receive late 
  if node.logs[prepare.n].cert_prepare_vote[prepare.i] != 1:
  	node.vote_prepare[prepare.i] = 1
  	node.logs[prepare.n].cert_prepare_num += 1
  	// if meet 2f + 1  go to commit phase
  	if node.logs[prepare.n].cert_prepare_num == 2f + 1:
  		node.logs[prepare.n].status = PREPARED
  		node.logs[prepare.n].cert_commit_num += 1
  		node.logs[prepare.n].cert_commit_vote[I_am] = 1
  		node.client_request[c].status = PREPARED
  		multicast (msg, my_view, prepare.n, prepare.d, I_am, COMMIT)
  		go to commit phase
  
```



### Commit

Backup and leader:  commit msg (msg, v, n, d, i, COMMIT)

```
assume msg's name is commit
When receive commit msg:
	//verify sender's identification
	if public key can't unwrap msg, return
	// check v
	if pre-prepare.view != pre-prepare.v: return
	// check h + L
	if n <= h or n > h + L: return
	
	// normal case
	if logs[commit.n].cert_commit_vote[commit.i] != 1:
    logs[commit.n].cert_commit_vote[commit.i] += 1
    logs[commit.n].cert_commit_num += 1
	if logs[commit.n].cert_commit_num == 2f + 1:
		logs[commit.n].status = commited
		//check apply pointer
		while apply + 1 <= h + L and logs[apply + 1].status == commited: appliy the log
		checkpoints_num = (apply - h) // k
		old_h = h
    h += checkpoints_num * k
    store the checkpoint between [old_h + 1 ~ new h]
		
```





#### VIEW-CHANGE:

When a backup finds a request is not completed in time, it starts view-change.

The main target of view-change is find a new leader, and transmit the work in previous view to new leader.

New leader get information of previous view from cluster, and assign number to them in new term.

TODO how to make leader election? why we need Q?

 ⟨VIEW-CHANGE, *v* + 1, *h*, C, P, Q, *i*⟩α*i*

```
When a backup finds system can't make progress:
	try to make leader election TODO
	only process view-change related msg, and omit other kinds of msgs.
	P, Q = list<n,d,v>, list<n,d,v>
	// create P(preparedm commited logs), Q(pre-prepared) set
	apply_flg = True // used in C
	for n, log in logs:
		if node.log.status == PREPARED or COMMITED, P.add(<log.n,log.d,log.v>)
		if node.log.status == PRE-PREPARED： Q.add(<log.n,log.d,log.v>)
		if node.log.status != APPLIED: apply_flg = False
    if apply_flg and n % k == 0: C.add(<n, d>)	
    delete pre-prepare, prepared and commit logs of its logs
    for k, v in node.client_request:
    // this pre-prepared msg would be droped in view change, so accept client's msg
    	if v.status == PRE-PREPARED: v.status = canceled  

	until receive NEW-VIEW msg: multicast(<VIEW-CHANGE, my_view, h, C, P, Q, i>) periodically
```

##### send VIEW-CHANGE-ACK:

why need d?

Since faulty replicas may send different VIEW-CHANGE msg and truy to break consensus. Leader should check digest. If replica A send VC to B with d, but send VC to Leader with d', leader would not think ACK is right until it receive 2f + 1 ack.

⟨VIEW-CHANGE-ACK,*v*+1,*i*, *j*,*d*⟩μ*ip*：

 *i* is the identifier of the sender, *d* is the digest of the VIEW-CHANGE message being acknowledged, and *j* is the replica that sent that VIEW-CHANGE message

all replicas 

```
assume view change msg = view_change
when receive view-change msg:
	check signature
	// check Q P
	for <n, d, v> in Q: if v > my_view： return
	for <n, d, v> in P: if v > my_view: return
	
	// normal operations
	cur_view_change_log = view_change_log(
			v = view_change.v,
			h = view_change.h
			C = view_change.C
			Q = view_change.Q
			i = view_change.i
			acks[I_am = 1, i = 1]
			ack_nums = 2
	)
	send <VIEW-CHANGE-ACK, v + 1, I_am, view_change.i, digest> to leader
	
```



#### when receive VIEW-CHANGE-ACK

⟨VIEW-CHANGE-ACK,*v*+1,*i*, *j*,*d*⟩μ*ip*

Leader:

```
// check something digest...可以在收到之前进行验证
when receive VIEW-CHANGE-ACK:
	if node.view_changes[j].acks[i] == False and node.view_change[j].digest = VIEW-CHANGE-ACK.digest:
		node.view_changes[j].acks[i] = True
		node.view_changes[j].ack_nums += 1
		if view_changes[j].ack_nums >= 2f + 1:
			node.view_change_num += 1
	if node.view_change_num >= 2f + 1:
		start process Decision procedure
```



#### Decision procedure TODO 同样的问题

all replicas:

```
//1. find h in C change to find low water mark h, and then <h', d> whose h' > h should be counted, when receive 2f + 1, h is determined
c_map = map<<h, d> : times>
start_h = -1
for view_change node.view_changes:
	for <h, d> in view_change.C:
		c_map[<h, d>] += 1
		if c_map[<h, d>] >= f + 1:
			start_h = max(start_h, h)

// may receive 3f + 1 msg so can be determined
//2. get correct <n, d> TODO, dif comp with paper and assign n with d
p_map = <<h, d, v> : times>
list<n, d> prepared size: L
for view_change node.view_changes:
	for <n, d, v> in view_change.P:
		if h <= n <= h + L - 1:
			p_map[<n, d, v>] += 1
			if p_map[<n, d, v>] += 1 >= f + 1:
				prepared[n % h] = <n,d>
for prepare_log in prepared:
	if prepare_log = Null:
		prepare_log = <n, NULL>
node.prepare = prepare_log
if I am leader:
	send <NEW-VIEW, v + 1, prepare>

```

#### Receive NEW-VIEW:

Backup:

```
when backup receive new-view msg:
	check node.my_prepare == new_view.prepare:
		if not, v + 1 and make new VIEW-CHANGE
		else: apply operations and go to the next h.
		
```



