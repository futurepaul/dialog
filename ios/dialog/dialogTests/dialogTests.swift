//
//  dialogTests.swift
//  dialogTests
//
//  Created by Paul Miller on 7/1/25.
//

import Testing

struct dialogTests {

    @Test("Basic functionality test")
    func example() async throws {
        // Basic test to verify testing framework works
        let testString = "Hello, World!"
        #expect(testString.count == 13)
        #expect(testString.contains("World"))
    }
    
    @Test("String operations test") 
    func testStringOperations() async throws {
        let message = "Test message for dialog app"
        #expect(!message.isEmpty)
        #expect(message.hasPrefix("Test"))
        #expect(message.hasSuffix("app"))
    }

}
