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
                (concatenate-string-list
                    (cons (intrinsic:concat-string-like-2 first second) more)))))


