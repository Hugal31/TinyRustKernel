#include <kstd.h>
#include <stdio.h>

void test_gettick(void)
{
  const unsigned long start = gettick();
  unsigned long last_print = 0;
  unsigned long delay = 0;
  while (delay < 4000) {
    if (delay - last_print >= 1000)
      {
	printf("%lu...\n", delay / 1000);
	last_print = delay;
      }
    delay = gettick() - start;
  }
}

void test_keyboard(void)
{
  puts("Try the arrow keys, or press ESC...");
  while (1) {
    switch (getkey())
      {
      case KEY_UP:
	puts("Up!");
	break;
      case KEY_LEFT:
	puts("Left!");
	break;
      case KEY_RIGHT:
	puts("Right!");
	break;
      case KEY_DOWN:
	puts("Down!");
	break;
      case KEY_ESC:
	return;
      default:
	break;
      }
  }
}

void entry(void)
{
  puts("Start");

  test_keyboard();
  test_gettick();

  puts("Stop");
  for (;;) {}
}
