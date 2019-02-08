#include <kstd.h>
#include <stdio.h>
#include <stdlib.h>

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

void test_filesystem(void)
{
    int fd;
    char buffer[1024];
    ssize_t readed;

    printf("Try to open a file... ");
    fd = open("text.txt", O_RDONLY);
    if (fd < 0)
    {
        puts("Failed...");
        return;
    }

    printf("Success ! fd is %d\nTry to read... ", fd);

    readed = read(fd, buffer, 1024 - 1);
    buffer[readed] = '\0';
    printf("Success ! Read \"%s\", len is %ld\n", buffer, readed);

    printf("Seek at the second character... %ld", seek(fd, 1, SEEK_SET));
    readed = read(fd, buffer, 1024 - 1);
    buffer[readed] = '\0';
    printf(": \"%s\"\n", buffer);

    printf("Close returned %d\n", close(fd));
}

void test_sbrk(void)
{
    int *i = malloc(sizeof(int));

    if (i == NULL) {
        puts("Could not alloc an int");
    }
    printf("Allocated ! address is 0x%p\n", i);

    free(i);
}

void test_video(void)
{
    if (setvideo(VIDEO_GRAPHIC) != 0) {
        puts("Cannot switch to video mode");
        return;
    }
}

void test_audio(void)
{
    struct melody *intro = load_sound("/intro.csf");

    if (intro == NULL) {
        puts("Could not load melody");
        return;
    }

    playsound(intro, -1);
}

void entry(void)
{
  puts("Start");

  test_filesystem();
  test_keyboard();
  test_gettick();

  puts("Stop");
  for (;;) {}
}
