(defun cons (car cdr)
    ;; TODO cdr should be a list or nil
    (intrinsic:cons car cdr))

(defun car (list)
    ;; TODO param should be a list
    (intrinsic:car list))

(defun cdr (list)
    ;; TODO param should be a list
    (intrinsic:cdr list))

(defun list (&rest items)
    items)

(defun cadr (list)
    (car (cdr list)))

(defun null (thingy)
    (if thingy nil t))

(defun append (&rest lists)
    (let (
        (first (car lists))
        (rest (cdr lists)))
        (if (null rest)
            first
            (apply #'append (cons (append-2 first (car rest)) (cdr rest))))))

;; this could maybe be private or letf
(defun append-2 (before after)
    (if (null before)
        after
        (if (null after)
            before
            (if (null (cdr before))
                ;; last item
                (cons (car before) after)
                ;; item before
                (cons (car before) (append-2 (cdr before) after))))))


(defun remove-if-not (func list)
    (if (null list)
        nil
        (if (funcall func (car list))
            (cons (car list) (remove-if-not func (cdr list)))
            (remove-if-not func (cdr list)))))
