#!/usr/bin/env hy

; This is optimized for Hy 0.12 running on Python3.5

(import select socket sys signal pwd os os.path)

(def BUFSIZE 1024)
(def MAXQ 1024)
(def CRLF "\r\n")

(defn get-plan-maybe [data]
  (let [sdata data
        rdata (cut sdata 2)
        end (.find rdata (.encode "\r\n"))]
    (while (= (get rdata 0) (.encode " ")) (setv rdata (cut rdata 1)))
    (try
     (let [username (cut rdata 1 end)
           entry (pwd.getpwnam (.decode username "utf-8"))]
       (if (and (os.path.isdir entry.pw_dir)
                (os.path.isfile (os.path.join entry.pw_dir ".plan")))
         (.encode (.read (open (os.path.join entry.pw_dir ".plan"))))
         None))
     (except [err KeyError] None))))


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
                    (.join "@" host name)))

        cleanup (fn [s]
                  (nonlocal clientmap)
                  (.close s)
                  (if (in s outputs)
                    (.remove outputs s))
                  (.remove inputs s)
                  (del clientmap s))]

    (.setsockopt server socket.SOL_SOCKET socket.SO_REUSEADDR 1)
    (.bind server (, "" port))
    (.listen server backlog)
    (signal.signal signal.SIGINT sighandler)
    (while running
      (try
       (let [(, inputready outputready exceptready) (select.select inputs outputs [])]
         (for [s inputready]
           (if (= s server)
             (let [(, client address) (.accept server)]
               (nonlocal clients clientmap)
               (setv clients (+ clients 1))
               (.append inputs client)
               (.append outputs client)
               (assoc clientmap client {"address" address "data" (bytearray "" "utf-8") "response" ""}))
             (let [client (get clientmap s)
                   data (+ (get client "data") (.recv s BUFSIZE))
                   ldata (len data)]
               (if (> ldata MAXQ)
                 (cleanup s)
                 (if (and (> ldata 2)
                          (not (= -1 (.find data (.encode "\r\n")))))
                   (assoc client "response" (get-plan-maybe data)))))))
         (for [s outputready]
           (if (not (= s server))
             (let [response (get (get clientmap s) "response")]
               (if (> (len response) 0)
                 (.send s (bytearray response)))
               (cleanup s)))))
       (except [e select.error] (break))
       (except [e socket.error] (break))))))

(ruipserver 5 7979)
