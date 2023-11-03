a = 4;
b = 6;

if a == b {
  print a + b;
} else {
  b = false; // Type error, rebound variable to different type!
  print 0;
}

print b - a;
