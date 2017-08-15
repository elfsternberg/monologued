#!/usr/bin/env hy

(import select socket sys signal)

(def BUFSIZE 1024)
(def MAXQ 1024)
(def CRLF "\r\n")

(defn ruipserver [backlog port]
  (let [clients 0
        clientmap {}
        outputs []
        running 1
        server (socket.socket socket.AF_INET socket.SOCK_STREAM)
        inputs [server]
        
        sighandler (fn [signum frame]
                     (for [o outputs] (.close 0))
                     (.close server))
        
        getname (fn [client]
                  (let [info (get clientmap client)
                        (, host name) (, (-> (get info 0) (get 0)) (get info 1))]
                    (.join "@" host name)))]

    (.setsockopt server socket.SOL_SOCKET socket.SO_REUSEADDR 1)
    (.bind server (, "" port))
    (.listen server backlog)
    (signal.signal signal.SIGINT sighandler)
    (while running
      (print inputs outputs)
      (try
       (let [(, inputready outputready exceptready) (select.select inputs outputs [])]
         (for [s inputready]
           (if (= s server)
             (let [(, client address) (.accept server)]
               (nonlocal clients)
               (setv clients (+ clients 1))
               (.append inputs client)
               (.append outputs client)
               (assoc clientmap client {"address" address "data" "" "response" ""}))
             (let [client (get clientmap s)
                   data (+ (get client "data") (.recv s BUFSIZE))
                   ldata (len data)]
               (if (> ldata MAXQ)
                 (.close s)
                 (if (and (> ldata 2) (= (cut data -2) CRLF))
                   (assoc client "response" (get-plan-maybe data)))))))
         (for [s outputready]
           (if (not (= s server))
             (let [response (get (get clientmap s) "response")]
               (if (> (len response) 0)
                 (.send s response)
                 (.close s))))))
       (except [e select.error] (break))
       (except [e socket.error] (break))))))

(ruipserver 5 7979)
