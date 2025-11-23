Feature: Interactions
  In order to control processes over different connections
  As a hacker
  I need to define a generic interface to a running process

  Scenario: Read chunk
    Given an Interaction with a 50 millisecond timeout
      | first  | .5 |
      | second |  3 |
      | third  |    |
      # ^ to send
      #         ^ * TIMEOUT to wait
    When I read a chunk
    Then the chunk I read should equal "firstsecond"

  Scenario: Read last chunk
    Given an Interaction with a 50 millisecond timeout
      | foobar | 3 |
      | foobaz |   |
    When I read the last chunk
    Then the chunk I read should equal "foobar"
