(let ((lambda (lambda () (format t "Hey there I'm in a lambda"))))
    (funcall lambda))

(let ((lambda (lambda (arg) (intrinsic:princ arg))))
    (funcall lambda "test with argument"))
