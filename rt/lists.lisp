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
