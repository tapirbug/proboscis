

(defun add-3-curried (a)
    ;;(dump "I'm inside a lambda")
    (lambda (b)
        ;;(dump "I'm inside a lambda")
        (lambda (c)
            (+ a b c))))

(defun add-2-curried (a)
    (lambda (b)
        (+ a b)))

(dump 100 "value of 100 in main")

(dump (funcall (add-2-curried 100) 1000))

(dump (funcall (funcall (add-3-curried 10) 20) 30))


;; add-3-curried:
;;     make-persistent-places count:123 to:local0
;;     consume-param to: local0[0]
;;     make-lambda stack:local0 function:add-3-curried#lambda1 to: local0[1]
;;     return value:local0[1]
;;
;; add-3-curried#lambda1:
;;     load-persistent-places to:local0 ;; this loads the stack thing in the function
;;     consume-param to: local0[2]
;;     make-lambda stack:local0 function:add-3-curried#lambda2 to: local0[3]
;;     return value:local0[3]
;;
;; add-3-curried#lambda2:
;;     load-persistent-places to:local0
;;     consume-param to: local0[4]
;;     ;; call add with local0[0] local0[2] and local0[4] and return it

;; local0 can be a parameter that all functions have I think, so maybe something like

;; add-3-curried:
;;      ;; it's implied here that persistent space has to be reserved
;;    consume-param to persistent0
;;    make-lambda function:add-3-curried#lambda1 to: persistent1 ;; (implicitly takes the current persistent space as argument)
;;    return value:persistent1
;;
;; add-3-curried#lambda1:
;;       ;; it's implied here that persistent space does not have to be reserved
;;    consume-param to persistent2
;;    make-lambda function:add-3-curried#lambda2 to: persistent3
;;    return persistent3
