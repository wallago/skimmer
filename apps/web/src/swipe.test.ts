import { expect, test } from "vitest";
import { MIN_SWIPE_PX, swipeAction } from "./swipe";

test("a long drag right reads, left keeps for later", () => {
  expect(swipeAction(MIN_SWIPE_PX, 0)).toBe("read");
  expect(swipeAction(-MIN_SWIPE_PX, 10)).toBe("later");
});

test("a short drag or a scroll does nothing", () => {
  expect(swipeAction(MIN_SWIPE_PX - 1, 0)).toBeNull();
  expect(swipeAction(100, 60)).toBeNull();
  expect(swipeAction(0, 0)).toBeNull();
});
