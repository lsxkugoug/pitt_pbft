
## view-change & retrainsmission(rt) & state transfer(st)
view-change, msg retrainsmission and state transfer related very closely and it is the most complicated part. This is my design for it.
### 0. node related data structure
```python
node {
    ...
    last_applied # record last applied sequence number
    # for rt and state transfer and pp restore
    # record other nodes' state, only accept next state larger than me
    restore_state map<digest, (count,byte[])>
    restore_state_vote: list[bool] length: server_num
    restore_pp map<seq, map<digest, (count, client_resquest)>>
    restore_pp_vote map<seq, list[bool] length:server_num>
    last_new_vew
    last_vc
    # record every node's last applied sequence number
    quorm_applied: int[server_num]
    # for view change
    vc_msgs: vec<VC> length: server_nums
    rts: vec<rt_msg> length: server_nums
    new_view: new_view_msg
}
```
### 1 explaination for overview design

First of all, rt message has two responsibility:
(1) msg retransmission
(2) broadcast my status, like last applied sequence number, which can help others to gc
(3) since everyone knows other last applied sequence number, we can build state transfer on it.

**Sender part:**
every node periodically send rt msg, which contains 

data structure
```python
rt_msg {
    # view: int
    who_send: int
    server_status: int # VIEW-CHANGE, NORMAL
    last_applied_seq: int
    log_entry_status_set: list<(seq: int, status: int, digest:byte[])>[last_applied_seq~L]
}
```

**receiver part:**
1. restore prepare and commit msg
2. do gc
3. judge wether I should send new-view, view-change, rt_rpl_msg.restore_state or rt_rpl_msg.restore_pp msg
```python
when receive rt_msg:
    # 1. get quorum_cp, do gc, wether do veiw change
    is_lag = false
    node.quorm_applied[who_send] = max(msg.last_applied_seq, node.quorm_applied[who_send])
    tmp_apl = sort(node.quorm_applied)
    quorum_h = node.quorm_applied[f] / K * K
    do_gc(quorum_h) # truncate log_entries before quorum_h
    if quorum_h > node.h + L:
        is_lag = true
        
    # 2. get valid log_entry_status and restore prepare or commit msg
    log_entry_init_set = []
    for idx, (status, digest) in log_entry_status_set:
        seq = msg.last_applied_seq + idx
        if status == PREPARED and node.h + L >= seq >= node.h:
            if not node.log.get(seq).prepare_vote[who_send] and 
            node.log.get(seq).digest == digest:
                node.log.get(seq).prepare_vote[who_send] = True
                node.log.get(seq).prepare_num += 1
                if node.log.get(seq).prepare_num > 2f:
                    node.log.get(seq).status = max(PREPARED, node.log.get(seq).status)
        if status == COMMITED node.h + L >= seq >= node.h:
            if not node.log.get(seq).commit_vote[who_send] and 
            node.log.get(seq).digest == digest:
                node.log.get(seq).commit_vote[who_send] = True
                node.log.get(seq).commit_num += 1
                if node.log.get(seq).commit_num > 2f:
                    node.log.get(seq).status = max(COMMITED, node.log.get(seq).status)
            if node.log.get(seq).status == COMMITED: try_apply()# this function tries to apply all commited logs
        if status == LOG_ENTRY_INIT:
            log_entry_init_set.add(seq)
    # 3.send view change, new view, rt_reply_msg
    # 3.1 construct rt_rpl_msg
    msg_state_seq = msg.last_applied_seq / L - 1
    my_state_seq = node.last_applied_seq / L - 1
    
    if my_state_seq > msg_state_seq:
        # only need state transfer byte
        rt_rpl_msg = {
            restore_state = node.get_state_byte(msg_state_seq + 1)
        }
        send(rt_rpl_msg)
    else:
        # only need pp
        pp_set = []
        for seq in log_entry_init_set:
            if node.log.get(seq) != None:
                pp_set.append((seq, client_request))
        rt_rpl_msg = {
            restore_state = node.get_state_byte(msg_state_seq + 1)
        }
        send(rt_rpl_msg)        

    # 3.2 if sender is do view change, send new-view, view-change
    if msg.status == PENDING:
        if I am leader:
            send NEW-VIEW msg
        else:
            send VIEW-CHANGE msg
    if is_lag:
        do_vc()
```
**reply data structure**
reply msg used to restore sender's state and pp msg
```python3
rt_rpl_msg {
    who_send: int
    # restore_state is empty or restore_pp is empty
    restore_state: (state_seq ,byte[]) # only for sender who need st
    restore_pp: list<seq, client_msg> # only for sender already complete state transfer
}
```

**code:**
when receive rt_rpl_msg:
```python
check part
check signature
if restore_state.state_seq!= None:
    check restore_state.state_seq == my_cur_state_seq
    # chech wether already vote
    if not node.restore_state_vote[msg.who_send]: return False

if restore_state.restore_pp!= None:
    check signature
    check wether corresponding slot is empty
    check wether sender has already vote for it.
do part:
# only has restore_state or restore_pp
if restore_state != None:
    # recover state
    node.restore_state[msg.who_send] = True
    node.restore_state<digest(msg.client_request)> = (count + 1, byte[])
    if count == 2f + 1:
        install state
        clear node.restore_state and node.restore_state_vote
        try_gc, slid the window
else:
    # recover pp
    # restore_pp map<seq, map<digest, (count, client_resquest)>>
    # restore_state_vote map<seq, list[bool] length:server_num>
    for pp in msg.restore_pp:
        if pp.seq not in h~h + L: continue
        node.restore_pp[seq][digest] = (count + 1, client_resquest)
        if count == 2f + 1；
            node.log.get(seq) = log_entry{Client.request}
            clear node.restore_pp[seq]
    
    
```


### view change example
j reply rt msg with two important datastructure:  
(1). state transfer byte  
(2). retransmission

if j is in view change state:
j only send state transfer byte, and send view change to i. In this time, if i lags, it must go state transfer, after it successfully transfer to quorum_cp, it can jump to current quorum_view.

**For example**, when quorum do view change:  
A 5->10 last_applied = 99, assume h = 90, L = 10  
B 5->10 last_applied = 99, assume h = 90, L = 10   
C 5->10 last_applied = 99, assume h = 90, L = 10  
D 1 last_applied = 99 = 1, h = 0
Process:
(1) Firstly, every node broadcast \<rt, status = pending>.
(2) when D find it lags(because of rt.last applied), it go to view change phase. Meanwhile, A B C find D lags, so it send /<rt-reply, state byte>.
(3) D send \<VIEW-CHANGE> with v + 1 view, and stuck until it receive 2f + 1 valid VC msg. Other send D with their VC or NEW VIEW because of D's pendingh rt. The leader also send the NEW-VIEW msg since it receives D's pending rt msg. Besides, others send D last VC msg. In this case,  
(4) After D receive 2f + 1 valid VC msg, start timer
(5) D wait NEW-VIEW msg until timer expires. If get good NEW-VIEW msg, status -> normal.
(6) D can make state transfer even in VIEW CHANGE, but it only change its view in VIEW CHANGE process.
(7) Liveness Rule2: When D recieve f + 1 VC msg, sort the view, and D can jump to f + 1 largest view. For example, view, A: 7, B: 8, C:8. D can directly jump to 8. Since 8 8 7, index(f + 1) = 8, change to 8, even the state is not catch up.

**For example,** when quorum is in normal phase:  
A 10 last_applied = 99, assume h = 90, L = 10  
B 10 last_applied = 99, assume h = 90, L = 10   
C 10 last_applied = 99, assume h = 90, L = 10  
D 1 last_applied = 1, h = 0

process:
(1) D finds it lags, so go to VIEW CHANGE.
(2) A B C receive D's rt. A B C send their last VC msg to D since D's rt msg status == VIEW CHANGE. 
(3) In this time D receives f + 1 valid VC msg whose view > D.view. So based on Liveness rule 1, it jump to view 10. Meanwhile, the leader also send the NEW-VIEW msg since it receives D's pending rt msg. Besides, others send D last VC msg. In this case, if NEW-VIEW is valid and NEW-VIEW.view == myv_view. D become normal node.

### Some corner case
1.Assume D go to VIEW CHANGE but actually it is in the correct view.
(1) D stucks and wait 2f + 1 VC msg whose view = D.v+1, but it can never collect enough certificates. Since quorum is good now.
(2) However, D's state is good since msg retransmission and state trransfer.
(3) When quorum cant make progress without D at some time. Quorum will go to VIEW CHANGE and transfer to v + 1 which is D's current view.





VIEW CHANGE msg structure:
```python3
vc_msg {
    msg_type: int
    view: int
    who_send: int
    last_stable_sq: int
    stable_cetificates: rt_msg[server_nums] with sig
    prepared_entries: vec<log_entry>
}
```

NEW VIEW msg structure:
```python3
new_view_msg {
    msg_type: int
    view: int
    VC_certificate: vec<vc_msg>
    state_set: Vec<state>
}
```





**code:**
When receive vc_msg
```python3
check part:
    0. check wether node.status == VIEW-CHANGE
    1. verify signature
    2. check vc_msg.view >= my_view + 1 # since i send v + 1 VC, and stuck until get 2f + 1 valid VC

store vc_msg to node.vc_msgs based on max(view, last_stable_sq)
```

when go to view change
```python3
if node.status == VIEW CHANGE: return
node.status = VIEW CHANGE
while 1 {
    # build VC msg

    for log_entry in node.logs:
        if log_entry.status >= prepared:
            prepared.append(log_entry)
    vc = vc_msg {
        msg_type: VIEW_CHANGE
        who_send: node.who_iam
        view: node.view + 1
        last_stable_sq: node.last_applied / L * L
        stable_cetificates: node.quorum_h
        prepared_entry: vec<log_entry>
    }
    node.vc_msgs[node.who_iam] = vc
    quorum_view = -1
    # Liveness rule 1: wait VC msg whose view >= v + 1
    while node.vc_msgs.length < 2f + 1 {
        vc.v = node.my_view + 1
        # liveness rule two: when it receive f + 1
        if node.vc_msgs.length > f + 1 {
            views = set of node.vc_msgs.view
            sort(views, reversed)
            quorum_view = views[f + 1]
            vc.v = max(vc.v, quorum_view)
        }
        broadcast(vc)
        thread.sleep(20 ms)
    }

    # now get enough vc to calculate new_view
    # todo later jobs after become leader or normal
    new_view = caculate_new_view()
    # if I am leader, broadcast new view
    if node.who_iam == quorum_view % server_num {
        broadcast(new_view)
        store node.last_view_msg = new_view
        node.view = quorum_view
        node.status = leader
        clear all log_entry
        late_leader_job() # send every new-view msg as pp
        break
    } else {
        while {
            success = False
            if node.new_view_msg == view_msg {
                node.my_view = view_msg.view
                node.status = NORMAL
                clear node.vc_msgs, node.new_view_msg
                success = True
                break
            }
            time_sleep(20ms)
        }
        if success:
            clear all log_entry
            break
    }
    
    # if I am normal node
    set timer *= 2
}
```

the algorithm calculate new view msg:
```python3
caculate_new_view() {
    # find valid vc set
    valid_vc_set: vec<VC>
    for msg in node.vc_msgs {
        if failed check signature:continue
        # check wether the h is correct
        success = True
        quorum_cp = vec<>
        for rt in msg.stable_cetificates {
            if failed check signature:
                success = False
                break
            quorum_cp.append(rt.last_applied_seq / K * K)
        }
        sort(quorum_cp, reverse)
        if msg.last_stable_sq != quorum_cp[f + 1]:
            success = False
        if not success: continue
        valid_vc_set.append(msg)
    }
    # find largest h
    valid_h = -1
    for vc in valid_vc_set:
        if vc.last_stable_sq > valid_h:
            valid_h = vc.last_stable_sq
    
    # use map to get correct (seq, digest)
    new_view_map = map[(seq, digest), count]
    for vc in valid_vc_set:
        for prepared_log_entry in vc.prepared_entries:
            if h + L >= prepared_log_entry.seq >= h:
                new_view_map[(prepared_log_entry.seq, prepared_log_entry.digest)] += 1
    
    new_view = vec<log_entry> length = L
    for (seq, digest), count in new_view_map:
        if count >= f + 1:
            new_view[seq - h] = digest
    return new_view
}   

```