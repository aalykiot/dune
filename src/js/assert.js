/**
 * Assert API
 *
 * Javascript, battle tested, simple assertion library with no dependencies.
 *
 * @see {@link https://assert-js.norbert.tech/}
 *
 * @module Assert
 */

const VALUE_NAME_REGEXP = /\${(.*?)}/g;

class MessageFactory {
  /**
   * @param {string} template
   * @param {object} [data]
   */
  static create(template, data = {}) {
    if (typeof template !== 'string') {
      throw new Error(
        `Expected string but got "${ValueConverter.toString(template)}".`
      );
    }

    if (typeof data !== 'object') {
      throw new Error(
        `Expected string but got "${ValueConverter.toString(data)}".`
      );
    }

    return template.replace(
      VALUE_NAME_REGEXP,
      function (placeholder, propertyName) {
        if (data.hasOwnProperty(propertyName)) {
          return data[propertyName];
        }

        return placeholder;
      }
    );
  }
}

class ValueConverter {
  /**
   * @param {*} value
   * @returns {string}
   */
  static toString(value) {
    if (typeof value === 'string') {
      return `string["${value}"]`;
    }

    if (typeof value === 'number') {
      if (Number.isInteger(value)) {
        return `int[${value}]`;
      }

      return `float[${value}]`;
    }

    if (typeof value === 'boolean') {
      return `boolean[${value ? 'true' : 'false'}]`;
    }

    if (typeof value === 'function') {
      return `function[${value.toString()}]`;
    }

    if (typeof value === 'object') {
      if (Array.isArray(value)) {
        return `array[length: ${value.length}]`;
      }

      if (value instanceof Map) {
        return `Map[size: ${value.size}]`;
      }

      if (value instanceof WeakMap) {
        return `WeakMap[]`;
      }

      if (value instanceof Set) {
        return `Set[size: ${value.size}]`;
      }

      if (value instanceof WeakSet) {
        return `WeakSet[]`;
      }

      if (value instanceof String) {
        return `String["${value}"]`;
      }

      if (value instanceof Number) {
        let source = value.valueOf();

        if (Number.isInteger(source)) {
          return `Number:int[${source}]`;
        }

        return `Number:float[${source}]`;
      }

      if (value instanceof Boolean) {
        return `Boolean[${value.valueOf() ? 'true' : 'false'}]`;
      }

      if (value instanceof Date) {
        return `Date["${value.toUTCString()}"]`;
      }

      if (value instanceof RegExp) {
        return `RegExp[${value.toString()}]`;
      }

      return `object[${JSON.stringify(value)}]`;
    }

    if (typeof value === 'undefined') {
      return 'undefined';
    }

    throw `Unhandled type ${typeof value}`;
  }
}

class InvalidValueException {
  /**
   * @param {string} type
   * @param {*} value
   * @param {string} [message]
   * @returns {Error}
   */
  static expected(type, value, message = '') {
    if (typeof message !== 'string') {
      throw new Error(
        `Expected string but got "${ValueConverter.toString(message)}".`
      );
    }

    if (message.length) {
      return new Error(
        MessageFactory.create(message, {
          expected: type,
          received: ValueConverter.toString(value),
        })
      );
    }

    return new Error(
      `Expected ${type} but got "${ValueConverter.toString(value)}".`
    );
  }
}

/**
 * A class that exposes static methods for assertions.
 */
class Assert {
  /**
   * Asserts that a given object is an instance of a specified class.
   *
   * @param {object} objectValue - The object to be tested against the expected instance.
   * @param {function} expectedInstance - The constructor function that the value is expected to be an instance of.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static instanceOf(objectValue, expectedInstance, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.instanceOf needs to be a valid string.'
    );

    if (typeof objectValue !== 'object') {
      throw InvalidValueException.expected('object', objectValue, message);
    }

    if (!(objectValue instanceof expectedInstance)) {
      throw InvalidValueException.expected(
        expectedInstance.name,
        objectValue,
        message.length
          ? message
          : 'Expected instance of "${expected}" but got "${received}".'
      );
    }
  }

  /**
   * Validates that a given value is an integer.
   *
   * @param {int} integerValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static integer(integerValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.integer needs to be a valid string.'
    );

    if (!Number.isInteger(integerValue)) {
      throw InvalidValueException.expected('integer', integerValue, message);
    }
  }

  /**
   * Validates that a given value is a number.
   *
   * @param {number} numberValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static number(numberValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.number needs to be a valid string.'
    );

    if (typeof numberValue !== 'number') {
      throw InvalidValueException.expected('number', numberValue);
    }
  }

  /**
   * Validates that a given value is a string.
   *
   * @param {string} stringValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static string(stringValue, message = '') {
    if (typeof message !== 'string') {
      throw new Error(
        'Custom error message passed to Assert.string needs to be a valid string.'
      );
    }

    if (typeof stringValue !== 'string') {
      throw InvalidValueException.expected('string', stringValue, message);
    }
  }

  /**
   * Validates that a given value is a boolean.
   *
   * @param {boolean} booleanValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static boolean(booleanValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.boolean needs to be a valid string.'
    );

    if (typeof booleanValue !== 'boolean') {
      throw InvalidValueException.expected('boolean', booleanValue, message);
    }
  }

  /**
   * Validates that a given value is true.
   *
   * @param {boolean} value - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static true(value, message = '') {
    this.boolean(value);
    this.string(
      message,
      'Custom error message passed to Assert.true needs to be a valid string.'
    );

    if (value !== true) {
      throw InvalidValueException.expected('true', value, message);
    }
  }

  /**
   * Validates that a given value is false.
   *
   * @param {boolean} value - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static false(value, message = '') {
    this.boolean(value);
    this.string(
      message,
      'Custom error message passed to Assert.false needs to be a valid string.'
    );

    if (value !== false) {
      throw InvalidValueException.expected('false', value, message);
    }
  }

  /**
   * Asserts that a given value is equal to an expected value.
   *
   * @param {*} value - The value to be compared. This can be of any type.
   * @param {*} expectedValue - The value against which the first parameter is compared.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static equal(value, expectedValue, message = '') {
    if (typeof value !== 'object') {
      this.true(
        value === expectedValue,
        message
          ? message
          : `Expected value ${ValueConverter.toString(
              value
            )} to be equals ${ValueConverter.toString(
              expectedValue
            )} but it's not.`
      );
    } else {
      this.objectEqual(
        value,
        expectedValue,
        message
          ? message
          : `Expected value ${ValueConverter.toString(
              value
            )} to be equals ${ValueConverter.toString(
              expectedValue
            )} but it's not.`
      );
    }
  }

  /**
   * Asserts that two objects are equal by comparing their properties.
   *
   * @param {object} object - The object to be compared.
   * @param {object} expectedObject - The object expected to be equal to the first object.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static objectEqual(object, expectedObject, message = '') {
    this.object(object, message);
    this.object(expectedObject, message);

    let objectProperties = Object.getOwnPropertyNames(object);
    let expectedObjectProperties = Object.getOwnPropertyNames(expectedObject);

    this.true(
      objectProperties.length === expectedObjectProperties.length,
      message
        ? message
        : `Expected object ${ValueConverter.toString(
            object
          )} to be equals ${ValueConverter.toString(
            expectedObject
          )} but it's not.`
    );

    objectProperties.forEach((objectProperty) => {
      this.equal(
        object[objectProperty],
        expectedObject[objectProperty],
        message
          ? message
          : `Expected object ${ValueConverter.toString(
              object
            )} to be equals ${ValueConverter.toString(
              expectedObject
            )} but it's not.`
      );
    });
  }

  /**
   * Asserts that a given value is of type 'object'.
   *
   * @param {object} objectValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static object(objectValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.object needs to be a valid string.'
    );

    if (typeof objectValue !== 'object') {
      throw InvalidValueException.expected('object', objectValue, message);
    }
  }

  /**
   * Asserts that a given object has a function of a specified name.
   *
   * @param {string} expectedFunctionName - The name of the function expected to be present in the object.
   * @param {object} objectValue - The object to be checked for the presence of the specified function.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static hasFunction(expectedFunctionName, objectValue, message = '') {
    this.string(expectedFunctionName);
    this.object(objectValue);
    this.string(
      message,
      'Custom error message passed to Assert.hasFunction needs to be a valid string.'
    );

    if (typeof objectValue[expectedFunctionName] !== 'function') {
      throw InvalidValueException.expected(
        `object to has function "${expectedFunctionName}"`,
        objectValue,
        message
      );
    }
  }

  /**
   * Asserts that a given object has a specific property.
   *
   * @param {string} expectedPropertyName - The name of the property expected to be present in the object.
   * @param {object} objectValue - The object to be checked for the presence of the specified property.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static hasProperty(expectedPropertyName, objectValue, message = '') {
    this.string(expectedPropertyName);
    this.object(objectValue);
    this.string(
      message,
      'Custom error message passed to Assert.hasProperty needs to be a valid string.'
    );

    if (typeof objectValue[expectedPropertyName] === 'undefined') {
      throw InvalidValueException.expected(
        `object to has property "${expectedPropertyName}"`,
        objectValue,
        message
      );
    }
  }

  /**
   * Asserts that a given value is an array.
   *
   * @param {array} arrayValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static array(arrayValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.array needs to be a valid string.'
    );

    if (!Array.isArray(arrayValue)) {
      throw InvalidValueException.expected('array', arrayValue, message);
    }
  }

  /**
   * Asserts that a given value is a function.
   *
   * @param {function} functionValue - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static isFunction(functionValue, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.isFunction needs to be a valid string.'
    );

    if (typeof functionValue !== 'function') {
      throw InvalidValueException.expected('function', functionValue, message);
    }
  }

  /**
   * Asserts that a given integer value is greater than an expected integer value.
   *
   * @param {int} expected - The integer value that the integerValue is expected to be greater than.
   * @param {int} integerValue - The integer value to be tested against the expected value.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static greaterThan(expected, integerValue, message = '') {
    this.number(expected);
    this.number(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.greaterThan needs to be a valid string.'
    );

    if (integerValue <= expected) {
      throw new Error(
        message.length > 0
          ? message
          : `Expected value ${integerValue} to be greater than ${expected}`
      );
    }
  }

  /**
   * Asserts that a given integer value is greater or equal than an expected integer value.
   *
   * @param {int} expected - The integer value that the integerValue is expected to be greater or equal than.
   * @param {int} integerValue - The integer value to be tested against the expected value.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static greaterThanOrEqual(expected, integerValue, message = '') {
    this.number(expected);
    this.number(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.greaterThanOrEqual needs to be a valid string.'
    );

    if (integerValue < expected) {
      throw new Error(
        message.length > 0
          ? message
          : `Expected value ${integerValue} to be greater than ${expected} or equal`
      );
    }
  }

  /**
   * Asserts that a given integer value is less than an expected integer value.
   *
   * @param {int} expected - The integer value that the integerValue is expected to be less than.
   * @param {int} integerValue - The integer value to be tested to ensure it is less than the expected value.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static lessThan(expected, integerValue, message = '') {
    this.number(expected);
    this.number(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.lessThan needs to be a valid string.'
    );

    if (integerValue >= expected) {
      throw new Error(
        message.length > 0
          ? message
          : `Expected value ${integerValue} to be less than ${expected}`
      );
    }
  }

  /**
   * Asserts that a given integer value is less or equal than an expected integer value.
   *
   * @param {int} expected - The integer value that the integerValue is expected to be less or equal than.
   * @param {int} integerValue - The integer value to be tested against the expected value.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static lessThanOrEqual(expected, integerValue, message = '') {
    this.number(expected);
    this.number(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.lessThanOrEqual needs to be a valid string.'
    );

    if (integerValue > expected) {
      throw new Error(
        message.length > 0
          ? message
          : `Expected value ${integerValue} to be less than ${expected} or equal`
      );
    }
  }

  /**
   * Asserts that the length of a given array matches an expected count.
   *
   * @param {int} expectedCount - The number that the length of the array is expected to match.
   * @param {array} arrayValue - The array whose length is being checked against the expected count.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static count(expectedCount, arrayValue, message = '') {
    this.integer(expectedCount);
    this.array(arrayValue);
    this.string(
      message,
      'Custom error message passed to Assert.count needs to be a valid string.'
    );

    if (arrayValue.length !== expectedCount) {
      throw new Error(
        message.length
          ? message
          : `Expected count ${expectedCount}, got ${arrayValue.length}`
      );
    }
  }

  /**
   * Asserts that a given value is not empty.
   *
   * @param {*} value - The value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static notEmpty(value, message = '') {
    this.string(
      message,
      'Custom error message passed to Assert.empty needs to be a valid string.'
    );

    if (value.length === 0) {
      throw InvalidValueException.expected('not empty value', value, message);
    }
  }

  /**
   * Asserts that a given integer value is an odd number.
   *
   * @param {int} integerValue - The integer value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static oddNumber(integerValue, message = '') {
    this.integer(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.oddNumber needs to be a valid string.'
    );

    if (integerValue % 2 !== 1) {
      throw InvalidValueException.expected('odd number', integerValue, message);
    }
  }

  /**
   * Asserts that a given integer value is an even number.
   *
   * @param {int} integerValue - The integer value to be checked.
   * @param {string} [message] - A custom error message to be used if the assertion fails.
   */
  static evenNumber(integerValue, message = '') {
    this.integer(integerValue);
    this.string(
      message,
      'Custom error message passed to Assert.evenNumber needs to be a valid string.'
    );

    if (integerValue % 2 !== 0) {
      throw InvalidValueException.expected(
        'even number',
        integerValue,
        message
      );
    }
  }

  /**
   * Asserts that a function throws an error.
   *
   * @param {function} callback - The function expected to throw an error when invoked.
   * @param {object} [expectedError] - An Error object representing the expected error.
   */
  static throws(callback, expectedError = new Error()) {
    this.isFunction(callback);

    try {
      callback();
    } catch (error) {
      if (
        typeof error === 'object' &&
        error instanceof Error &&
        typeof expectedError === 'object' &&
        expectedError instanceof Error
      ) {
        if (expectedError.message.length) {
          this.equal(
            error.message,
            expectedError.message,
            `Expected exception message "${error.message}" to be equals "${expectedError.message}" but it's not.`
          );
        }

        return;
      }

      this.equal(
        error,
        expectedError,
        `Expected error of type ${ValueConverter.toString(
          error
        )} to be equals ${ValueConverter.toString(expectedError)} but it's not.`
      );

      return;
    }

    throw InvalidValueException.expected(
      ValueConverter.toString(expectedError),
      null,
      'Expected from callback to throw an Error "${expected}" but it didn\'t.'
    );
  }
}

export default Assert;
