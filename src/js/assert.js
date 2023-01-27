// Assert API
//
// Javascript, battle tested, simple assertion library with no dependencies.
// https://assert-js.norbert.tech/

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

class Assert {
  /**
   * @param {object} objectValue
   * @param {function} expectedInstance
   * @param {string} [message]
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
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {number} numberValue
   * @param {string} [message]
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
   * @param {string} stringValue
   * @param {string} [message]
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
   * @param {boolean} booleanValue
   * @param {string} [message]
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
   * @param {boolean} value
   * @param {string} [message]
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
   * @param {boolean} value
   * @param {string} [message]
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
   * @param value
   * @param expectedValue
   * @param {string} [message]
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
   * @param {object} object
   * @param {object} expectedObject
   * @param {string} [message]
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
   * @param {object} objectValue
   * @param {string} [message]
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
   * @param {string} expectedFunctionName
   * @param {object} objectValue
   * @param {string} [message]
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
   * @param {string} expectedPropertyName
   * @param {object} objectValue
   * @param {string} [message]
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
   * @param {array} arrayValue
   * @param {string} [message]
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
   * @param {function} functionValue
   * @param {string} [message]
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
   * @param {int} expected
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {int} expected
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {int} expected
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {int} expected
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {int} expectedCount
   * @param {array} arrayValue
   * @param {string} [message]
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
   * @param {*} value
   * @param {string} [message]
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
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {int} integerValue
   * @param {string} [message]
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
   * @param {function} callback
   * @param {object} [expectedError]
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
