Feature: Byte-string manipulation
  In order to construct complex byte-strings
  As a hacker
  I want convenience functions for common byte-string manipulations

  Scenario: Hex bytes
    Given the unsigned 32-bit hex value 0x10203040
    When I convert the unsigned 32-bit hex value to bytes
    Then I should have the bytes
      | 0x10 |
      | 0x20 |
      | 0x30 |
      | 0x40 |

  Scenario: repeat
    # 0x7f
    Given the byte-string ""
    When I repeat the first byte into an unsigned 64-bit integer
    Then the unsigned 64-bit integer should equal 0x7f7f7f7f7f7f7f7f

  Rule: padding

    Background:
      Given a final padded byte-string length of 32

    Scenario: Left-padded
      Given the byte-string "AAAA"
      When I pad the left side of the byte-string
      Then the sequence of the byte-string will be
        |   | 28 |
        | A |  4 |

    Scenario: Right-padded hex bytes
      Given the unsigned 32-bit hex value 0x12030
      When I convert the unsigned 32-bit hex value to bytes
      And I pad the right side of the byte-string with '1's
      Then the sequence of the byte-string will be
        | 0x01 |    |
        | 0x20 |    |
        | 0x30 |    |
        |    1 | 29 |

    Scenario: Pad both
      Given the byte-string "AAAA"
      When I pad the byte-string
      Then the sequence of the byte-string will be
        |   | 12 |
        | A |  4 |
        |   | 16 |
