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

(defn tap [a] (print a) a)

(defclass server []
  [--init-- (fn [self backlog port]
     (setv self.clients 0)
     (setv self.clientmap {})
     (setv self.outputs [])
     (setv self.running True)
     (setv self.server (socket.socket socket.AF_INET socket.SOCK_STREAM))
     (.setsockopt self.server socket.SOL_SOCKET socket.SO_REUSEADDR 1)
     (.bind self.server (, "" port))
     (.listen self.server backlog)
     (signal.signal signal.SIGINT self.sighandler))

   addclient (fn [self]
     (setv (, client address) (.accept self.server))
     (setv self.clients (+ self.clients 1))
     (.append self.outputs client)
     (assoc self.clientmap client
            {"address" address "data" (bytearray "" "utf-8") "response" ""}))

   getrequest (fn [self s]
     (setv client (get self.clientmap s))
     (setv data (+ (get client "data") (.recv s BUFSIZE)))
     (setv ldata (len data))
     (if (> ldata MAXQ)
       (.cleanup self s)
       (if (and (> ldata 2)
                (not (= -1 (.find data (.encode "\r\n")))))
         (assoc client "response" (get-plan-maybe data)))))

   serve (fn [self]
     (while self.running
       (do
         (setv inputs (+ [self.server] (list (.keys self.clientmap))))

         (try
           (setv (, inputready outputready exceptready) (select.select inputs self.outputs []))
           (except [e select.error] (do (print "SELECT.ERROR") (break)))
           (except [e socket.error] (do (print "SOCKET.ERROR") (break))))

         (for [s inputready]
           (if (= s self.server)
             (.addclient self)
             (.getrequest self s))

           (for [s outputready]
             (if (not (= s self.server))
               (do 
                 (setv response (get (get self.clientmap s) "response"))
                 (if (> (len response) 0)
                   (do
                     (.send s (bytearray response))
                     (.cleanup self s))))))))))

   cleanup (fn [self s]
     (.close s)
     (if (in s self.outputs)
       (.remove self.outputs s))
     (del (get self.clientmap s))
     (setv self.clients (- self.clients 1)))

   sighandler (fn [self signum frame]
     (for [o self.outputs]
       (.close o))
     (.close self.server)
     (setv self.running False))])

(.serve (server 5 7979))
