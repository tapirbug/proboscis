(defun list (&rest items)
    items)

(defun cadr (list)
    (car (cdr list)))

(defun null (thingy)
    (if thingy nil t))

(defun not (thingy)
    (if thingy nil t))

(defun + (arg &rest rest)
    (if (null rest)
        arg
        (+-list arg rest)))

(defun +-list (addend list)
    (if (null list)
        addend
        (--list (add-2 addend (car list)) (cdr list))))

(defun - (arg &rest rest)
    (if (null rest)
        (sub-2 0 arg)
        (--list arg rest)))

(defun --list (subtrahend list)
    (if (null list)
        subtrahend
        (--list (sub-2 subtrahend (car list)) (cdr list))))

(defun = (first &rest rest)
    (=-list first rest))

(defun =-list (first rest)
    (if (null rest)
        t
        (if (nil-if-0 (sub-2 first (car rest)))
            nil
            (=-list first (cdr rest)))))

(defun /= (first &rest rest)
    (not (=-list first rest)))

;;    (let ((result (sub-2 minuend subtrahend))
;;        (if (null more-subtrahends) result (- result)))))
(defun concatenate (type &rest strings)
    ;; we assume that the type is 'string without checking
    (concatenate-string-list strings))

(defun concatenate-string-list (list)
    (let (
        (first (car list))
        (second (cadr list))
        (more (cdr (cdr list))))
            (if (null second)
                first
                (concatenate-string-list (cons (concat-string-like-2 first second) more)))))