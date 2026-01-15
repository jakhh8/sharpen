using System;

namespace Example.Managed
{

	public class ExampleClass
	{

		private int MemberVar = 0;

		public ExampleClass(int memberVar)
		{
			MemberVar = memberVar;
		}

		public static float StaticMethod(float value)
		{
			Console.WriteLine($"Squaring {value} in C#");

			return value * value;
		}

		public void MemberMethod()
		{
			Console.WriteLine($"C# MemberVar: {MemberVar}");
		}

	}

}
